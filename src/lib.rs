use bitflags::bitflags;
use std::{
    ffi::{OsStr, OsString},
    os::windows::prelude::OsStringExt,
};
use std::{os::windows::ffi::OsStrExt, path::PathBuf};
use std::{
    path::Path,
    ptr::{self},
};
use winapi::um::{
    minwinbase::{SYSTEMTIME, WIN32_FIND_DATAW},
    winnt::MAXDWORD,
};
use winapi::um::{
    timezoneapi::FileTimeToSystemTime,
    winnt::{FILE_ATTRIBUTE_DIRECTORY, HANDLE},
};
use winapi::{shared::minwindef::DWORD, um::handleapi::INVALID_HANDLE_VALUE};
use winapi::{
    shared::minwindef::FILETIME,
    um::fileapi::{FindClose, FindFirstFileW, FindNextFileW},
};

bitflags! {
  #[derive(Debug, PartialEq, Clone, Default)]
   pub struct FileAttributes: DWORD {
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
pub struct Time {
    pub year: u16,
    pub month: u16,
    pub day_of_week: u16,
    pub day: u16,
    pub hour: u16,
    pub minute: u16,
    pub second: u16,
    pub milliseconds: u16,
}

impl Time {
    pub fn dmyhm(&self) -> String {
        format!(
            "{:02}/{:02}/{:04} {:02}:{:02}",
            self.day, self.month, self.year, self.hour, self.minute,
        )
    }
}

impl From<SYSTEMTIME> for Time {
    fn from(value: SYSTEMTIME) -> Self {
        Self {
            year: value.wYear,
            month: value.wMonth,
            day_of_week: value.wDayOfWeek,
            day: value.wDay,
            hour: value.wHour,
            minute: value.wMinute,
            second: value.wSecond,
            milliseconds: value.wMilliseconds,
        }
    }
}

pub fn system_time(file_time: FILETIME) -> Result<SYSTEMTIME, Error> {
    unsafe {
        let mut system_time: SYSTEMTIME = std::mem::zeroed();
        if FileTimeToSystemTime(&file_time, &mut system_time) != 0 {
            Ok(system_time)
        } else {
            Err(Error::InvalidSystemTime)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DirEntry {
    pub name: OsString,
    pub path: PathBuf,
    pub date_created: Time,
    pub last_access: Time,
    pub last_write: Time,
    pub attributes: FileAttributes,
    ///Size in bytes
    //TODO: Change to u64, folders can just have a size of 0.
    pub size: Option<u64>,
}

impl DirEntry {
    pub fn is_folder(&self) -> bool {
        self.attributes.contains(FileAttributes::DIRECTORY)
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidSearch(PathBuf),
    InvalidSystemTime,
}

pub fn walkdir<S: AsRef<Path>>(path: S, depth: usize) -> Vec<Result<DirEntry, Error>> {
    unsafe {
        let search_pattern_wide: Vec<u16> = OsStr::new(path.as_ref())
            .encode_wide()
            .chain(Some(b'\\' as u16).into_iter())
            .chain(Some(b'*' as u16).into_iter())
            .chain(Some(0).into_iter())
            .collect();

        let mut fd: WIN32_FIND_DATAW = std::mem::zeroed();
        let search_handle: HANDLE = FindFirstFileW(search_pattern_wide.as_ptr(), &mut fd);
        let mut files = Vec::new();

        if search_handle != ptr::null_mut() && search_handle != INVALID_HANDLE_VALUE {
            loop {
                let nul_range_end = fd
                    .cFileName
                    .iter()
                    .position(|&c| c == b'\0' as u16)
                    .unwrap_or_else(|| fd.cFileName.len());
                let name = OsString::from_wide(&fd.cFileName[..nul_range_end]);

                //Skip these results.
                if name == ".." || name == "." {
                    fd = std::mem::zeroed();
                    if FindNextFileW(search_handle, &mut fd) == 0 {
                        break;
                    }
                    continue;
                }

                let is_dir = (fd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0;

                //TODO: I'm fairly sure these dates are wrong.
                //Handle unwraps better.
                let date_created = Time::from(system_time(fd.ftCreationTime).unwrap());
                let last_access = Time::from(system_time(fd.ftLastAccessTime).unwrap());
                let last_write = Time::from(system_time(fd.ftLastWriteTime).unwrap());

                let attributes = FileAttributes::from_bits_truncate(fd.dwFileAttributes);
                let size =
                    (fd.nFileSizeHigh as u64 * (MAXDWORD as u64 + 1)) + fd.nFileSizeLow as u64;
                let size = if is_dir { None } else { Some(size) };

                //TODO: Path might not actually exist.
                let mut p = PathBuf::new();
                p.push(path.as_ref());
                p.push(name.clone());

                if is_dir {
                    if depth != 0 {
                        if depth - 1 != 0 {
                            files.extend(walkdir(p.as_path(), depth - 1));
                        }
                    } else {
                        files.extend(walkdir(p.as_path(), 0));
                    }
                }

                files.push(Ok(DirEntry {
                    name,
                    path: p,
                    date_created,
                    last_access,
                    last_write,
                    attributes,
                    size,
                }));

                fd = std::mem::zeroed();

                if FindNextFileW(search_handle, &mut fd) == 0 {
                    break;
                }
            }

            FindClose(search_handle);

            files
        } else {
            files.push(Err(Error::InvalidSearch(path.as_ref().to_path_buf())));
            files
        }
    }
}
