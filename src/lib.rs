use bitflags::bitflags;
use std::{
    ffi::{c_void, OsStr},
    mem::transmute,
    path::{Path, PathBuf},
    ptr::{self},
    slice::from_raw_parts,
};

pub const INVALID_HANDLE_VALUE: *mut c_void = -1isize as *mut c_void;
pub const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x00000010;

#[rustfmt::skip]
extern "system" {
    pub fn FileTimeToSystemTime(lpFileTime: *const FileTime, lpSystemTime: *mut SystemTime) -> bool;

    pub fn FindFirstFileA(lpFileName: *const i8, lpFindFileData: *mut FindDataA) -> *mut c_void;
    pub fn FindNextFileA(hFindFile: *mut c_void, lpFindFileData: *mut FindDataA) -> bool;

    pub fn FindClose(hFindFile: *mut c_void) -> bool;
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct FindDataA {
    pub file_attributes: u32,
    pub creation_time: FileTime,
    pub last_access_time: FileTime,
    pub last_write_time: FileTime,
    pub file_size_high: u32,
    pub file_size_low: u32,
    pub reserved0: u32,
    pub reserved1: u32,
    pub file_name: [i8; 260],
    pub alternate_file_name: [i8; 14],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct FileTime {
    dw_low_date_time: u32,
    dw_high_date_time: u32,
}

#[derive(Debug)]
pub enum Error {
    InvalidSearch(PathBuf),
    InvalidSystemTime,
}

impl TryInto<SystemTime> for FileTime {
    type Error = Error;

    fn try_into(self) -> Result<SystemTime, Self::Error> {
        unsafe {
            let mut system_time = SystemTime::default();
            if FileTimeToSystemTime(&self, &mut system_time) {
                Ok(system_time)
            } else {
                Err(Error::InvalidSystemTime)
            }
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct SystemTime {
    pub year: u16,
    pub month: u16,
    pub day_of_week: u16,
    pub day: u16,
    pub hour: u16,
    pub minute: u16,
    pub second: u16,
    pub milliseconds: u16,
}

impl SystemTime {
    pub fn dmyhm(&self) -> String {
        format!(
            "{:02}/{:02}/{:04} {:02}:{:02}",
            self.day, self.month, self.year, self.hour, self.minute,
        )
    }
}

bitflags! {
  #[derive(Debug, PartialEq, Clone, Default)]
   pub struct FileAttributes: u32 {
        const READONLY = 0x00000001;
        const HIDDEN = 0x00000002;
        const SYSTEM = 0x00000004;
        const DIRECTORY = 0x00000010;
        const ARCHIVE = 0x00000020;
        const DEVICE = 0x00000040;
        const NORMAL = 0x00000080;
        const TEMPORARY = 0x00000100;
        const SPARSE_FILE = 0x00000200;
        const REPARSE_POINT = 0x00000400;
        const COMPRESSED = 0x00000800;
        const OFFLINE = 0x00001000;
        const NOT_CONTENT_INDEXED = 0x00002000;
        const ENCRYPTED = 0x00004000;
        const INTEGRITY_STREAM = 0x00008000;
        const VIRTUAL = 0x00010000;
        const NO_SCRUB_DATA = 0x00020000;
        const EA = 0x00040000;
        const PINNED = 0x00080000;
        const UNPINNED = 0x00100000;
        const RECALL_ON_OPEN = 0x00400000;
        const RECALL_ON_DATA_ACCESS = 0x00400000;
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DirEntry {
    pub name: String,
    pub path: PathBuf,
    pub date_created: SystemTime,
    pub last_access: SystemTime,
    pub last_write: SystemTime,
    pub attributes: FileAttributes,
    ///Size in bytes
    //TODO: Change to u64, folders can just have a size of 0.
    pub size: Option<u64>,
}

impl DirEntry {
    pub fn extension(&self) -> Option<&'_ OsStr> {
        let mut iter = self.name.as_bytes().rsplitn(2, |b| *b == b'.');
        let after = iter.next();
        let before = iter.next();
        if before == Some(b"") {
            None
        } else {
            unsafe { after.map(|s| &*(s as *const [u8] as *const OsStr)) }
        }
    }
    pub fn is_folder(&self) -> bool {
        self.attributes.contains(FileAttributes::DIRECTORY)
    }
}

pub fn walkdir<S: AsRef<Path>>(path: S, depth: usize) -> Vec<Result<DirEntry, Error>> {
    unsafe {
        let path = path.as_ref();
        let search_pattern = [path.as_os_str().as_encoded_bytes(), &[b'\\', b'*', 0]].concat();

        let mut fd: FindDataA = std::mem::zeroed();
        let search_handle = FindFirstFileA(search_pattern.as_ptr() as *mut i8, &mut fd);
        let mut files = Vec::new();

        if search_handle != ptr::null_mut() && search_handle != INVALID_HANDLE_VALUE {
            loop {
                let end = fd
                    .file_name
                    .iter()
                    .position(|&c| c == b'\0' as i8)
                    .unwrap_or_else(|| fd.file_name.len());
                let slice = from_raw_parts(fd.file_name.as_ptr() as *const u8, end);
                let str: &str = transmute(slice);
                let name = str.to_string();

                //Skip these results.
                if name == ".." || name == "." {
                    fd = std::mem::zeroed();
                    if !FindNextFileA(search_handle, &mut fd) {
                        break;
                    }
                    continue;
                }

                let is_folder = (fd.file_attributes & FILE_ATTRIBUTE_DIRECTORY) != 0;

                //TODO: I think these dates are wrong.
                let date_created = fd.creation_time.try_into().unwrap();
                let last_access = fd.last_access_time.try_into().unwrap();
                let last_write = fd.last_write_time.try_into().unwrap();

                let attributes = FileAttributes::from_bits_truncate(fd.file_attributes);
                let size =
                    (fd.file_size_high as u64 * (u32::MAX as u64 + 1)) + fd.file_size_low as u64;
                let size = if is_folder { None } else { Some(size) };

                //TODO: Could be a faster way of getting path?
                let mut path = path.to_path_buf();
                path.push(&name);

                if is_folder {
                    if depth != 0 {
                        if depth - 1 != 0 {
                            files.extend(walkdir(&path, depth - 1));
                        }
                    } else {
                        files.extend(walkdir(&path, 0));
                    }
                }

                files.push(Ok(DirEntry {
                    name,
                    path,
                    date_created,
                    last_access,
                    last_write,
                    attributes,
                    size,
                }));

                fd = std::mem::zeroed();

                if !FindNextFileA(search_handle, &mut fd) {
                    break;
                }
            }

            FindClose(search_handle);

            files
        } else {
            files.push(Err(Error::InvalidSearch(path.to_path_buf())));
            files
        }
    }
}
