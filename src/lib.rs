use std::{
    ffi::{c_void, OsStr},
    mem::transmute,
    ptr::{self},
    slice::from_raw_parts,
};

const INVALID_HANDLE_VALUE: *mut c_void = -1isize as *mut c_void;

extern "system" {
    fn FileTimeToSystemTime(lpFileTime: *const FileTime, lpSystemTime: *mut SystemTime) -> bool;
    fn FindFirstFileA(lpFileName: *const i8, lpFindFileData: *mut FindDataA) -> *mut c_void;
    fn FindNextFileA(hFindFile: *mut c_void, lpFindFileData: *mut FindDataA) -> bool;
    fn FindClose(hFindFile: *mut c_void) -> bool;
    fn GetLogicalDrives() -> u32;
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct FindDataA {
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
struct FileTime {
    pub dw_low_date_time: u32,
    pub dw_high_date_time: u32,
}

#[derive(Debug)]
pub enum Error {
    InvalidSearch(String),
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
    /// Returns the date in day/month/year hour:minute format.
    pub fn dmyhm(&self) -> String {
        format!(
            "{:02}/{:02}/{:04} {:02}:{:02}",
            self.day, self.month, self.year, self.hour, self.minute,
        )
    }
}

/// File attributes are metadata values stored by the file system on disk.
///
/// [File Attribute Constants - MSDN](https://learn.microsoft.com/en-us/windows/win32/fileio/file-attribute-constants)
pub mod attributes {
    pub const READONLY: u32 = 0x00000001;
    pub const HIDDEN: u32 = 0x00000002;
    pub const SYSTEM: u32 = 0x00000004;
    pub const DIRECTORY: u32 = 0x00000010;
    pub const ARCHIVE: u32 = 0x00000020;
    pub const DEVICE: u32 = 0x00000040;
    pub const NORMAL: u32 = 0x00000080;
    pub const TEMPORARY: u32 = 0x00000100;
    pub const SPARSE_FILE: u32 = 0x00000200;
    pub const REPARSE_POINT: u32 = 0x00000400;
    pub const COMPRESSED: u32 = 0x00000800;
    pub const OFFLINE: u32 = 0x00001000;
    pub const NOT_CONTENT_INDEXED: u32 = 0x00002000;
    pub const ENCRYPTED: u32 = 0x00004000;
    pub const INTEGRITY_STREAM: u32 = 0x00008000;
    pub const VIRTUAL: u32 = 0x00010000;
    pub const NO_SCRUB_DATA: u32 = 0x00020000;
    pub const EA: u32 = 0x00040000;
    pub const PINNED: u32 = 0x00080000;
    pub const UNPINNED: u32 = 0x00100000;
    pub const RECALL_ON_OPEN: u32 = 0x00400000;
    pub const RECALL_ON_DATA_ACCESS: u32 = 0x00400000;
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub date_created: SystemTime,
    pub last_access: SystemTime,
    pub last_write: SystemTime,
    /// Bitflag for file [attributes].
    pub attributes: u32,
    /// Size in bytes.
    pub size: u64,
    pub is_folder: bool,
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
}

/// Traverse the requested directory.
///
/// A depth of `0` will set no limit.
///
/// ```
/// for file in winwalk::walkdir("D:\\Desktop", 1).into_iter().flatten() {
///     println!("Name: {}", file.name);
///     println!("Path: {}", file.path);
///     println!("Size: {}", file.size);
///     println!("Folder?: {}", file.is_folder);
///     println!("Last Write: {:?}", file.last_write);
///     println!("Last Access: {:?}", file.last_access);
///     println!("Attributes: {:?}", file.attributes);
/// }
/// ```
pub fn walkdir<S: AsRef<str>>(path: S, depth: usize) -> Vec<Result<DirEntry, Error>> {
    unsafe {
        let path = path.as_ref();
        let search_pattern = [path.as_bytes(), &[b'\\', b'*', 0]].concat();

        let mut fd: FindDataA = std::mem::zeroed();
        let search_handle = FindFirstFileA(search_pattern.as_ptr() as *mut i8, &mut fd);
        let mut files = Vec::new();

        if search_handle != ptr::null_mut() && search_handle != INVALID_HANDLE_VALUE {
            loop {
                //Create the full path.
                let end = fd
                    .file_name
                    .iter()
                    .position(|&c| c == b'\0' as i8)
                    .unwrap_or_else(|| fd.file_name.len());
                let slice = from_raw_parts(fd.file_name.as_ptr() as *const u8, end);
                let name: &str = transmute(slice);
                let path = [path, name].join("\\");

                //Skip these results.
                if name == ".." || name == "." {
                    fd = std::mem::zeroed();
                    if !FindNextFileA(search_handle, &mut fd) {
                        break;
                    }
                    continue;
                }

                let is_folder = (fd.file_attributes & attributes::DIRECTORY) != 0;

                //TODO: I think these dates are wrong.
                let date_created = fd.creation_time.try_into().unwrap();
                let last_access = fd.last_access_time.try_into().unwrap();
                let last_write = fd.last_write_time.try_into().unwrap();

                let size =
                    (fd.file_size_high as u64 * (u32::MAX as u64 + 1)) + fd.file_size_low as u64;

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
                    name: name.to_string(),
                    path,
                    date_created,
                    last_access,
                    last_write,
                    attributes: fd.file_attributes,
                    size,
                    is_folder,
                }));

                fd = std::mem::zeroed();

                if !FindNextFileA(search_handle, &mut fd) {
                    break;
                }
            }

            FindClose(search_handle);

            files
        } else {
            files.push(Err(Error::InvalidSearch(path.to_string())));
            files
        }
    }
}

/// Get the current system drives. `A-Z` `0-25`
pub fn drives() -> [Option<char>; 26] {
    let logical_drives = unsafe { GetLogicalDrives() };
    let mut drives = [None; 26];
    let mut mask = 1;

    for (i, letter) in (b'A'..=b'Z').enumerate() {
        if (logical_drives & mask) != 0 {
            drives[i] = Some(letter as char);
        }
        mask <<= 1;
    }

    drives
}
