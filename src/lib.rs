use bitflags::bitflags;
use std::os::windows::ffi::OsStrExt;
use std::ptr::{self};
use std::{
    ffi::{OsStr, OsString},
    os::windows::prelude::OsStringExt,
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
  #[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct FileTime {
    pub low: u32,
    pub high: u32,
}

impl FileTime {
    pub fn system_time(&self) -> Result<SYSTEMTIME, ()> {
        unsafe {
            let mut system_time: SYSTEMTIME = std::mem::zeroed();
            if FileTimeToSystemTime(
                &FILETIME {
                    dwLowDateTime: self.low,
                    dwHighDateTime: self.high,
                },
                &mut system_time,
            ) != 0
            {
                Ok(system_time)
            } else {
                Err(())
            }
        }
    }
}

impl From<FILETIME> for FileTime {
    fn from(val: FILETIME) -> Self {
        FileTime {
            low: val.dwLowDateTime,
            high: val.dwHighDateTime,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DirEntry {
    pub name: OsString,
    pub creation_time: FileTime,
    pub last_access: FileTime,
    pub last_write: FileTime,
    pub attributes: FileAttributes,
    ///Size in bytes
    pub size: Option<u64>,
}

impl DirEntry {
    pub fn is_dir(&self) -> bool {
        self.attributes.contains(FileAttributes::DIRECTORY)
    }
}

//TODO: Depth + Recursion.
//Options to ignore hidden and system files.
pub fn walkdir<S: AsRef<str>>(path: S) -> Result<Vec<DirEntry>, ()> {
    let search_pattern_wide: Vec<u16> = OsStr::new(path.as_ref())
        .encode_wide()
        .chain(Some(b'\\' as u16).into_iter())
        .chain(Some(b'*' as u16).into_iter())
        .chain(Some(0).into_iter())
        .collect();

    let mut fd: WIN32_FIND_DATAW = unsafe { std::mem::zeroed() };
    #[rustfmt::skip]
    let search_handle: HANDLE = unsafe { FindFirstFileW(search_pattern_wide.as_ptr(), &mut fd) };
    let mut files = Vec::new();

    if search_handle != ptr::null_mut() && search_handle != INVALID_HANDLE_VALUE {
        loop {
            let nul_range_end = fd
                .cFileName
                .iter()
                .position(|&c| c == b'\0' as u16)
                .unwrap_or(fd.cFileName.len());
            let name = OsString::from_wide(&fd.cFileName[..nul_range_end]);
            let is_dir = (fd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0;

            let creation_time = fd.ftCreationTime;
            let last_access = fd.ftLastAccessTime;
            let last_write = fd.ftLastWriteTime;

            let attributes = FileAttributes::from_bits_truncate(fd.dwFileAttributes);
            let size = (fd.nFileSizeHigh as u64 * (MAXDWORD as u64 + 1)) + fd.nFileSizeLow as u64;
            let size = if is_dir { None } else { Some(size) };

            files.push(DirEntry {
                name,
                creation_time: creation_time.into(),
                last_access: last_access.into(),
                last_write: last_write.into(),
                attributes,
                size,
            });

            fd = unsafe { std::mem::zeroed() };
            if unsafe { FindNextFileW(search_handle, &mut fd) == 0 } {
                break;
            }
        }

        unsafe {
            FindClose(search_handle);
        }

        Ok(files)
    } else {
        Err(())
    }
}
