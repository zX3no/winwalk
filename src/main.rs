use bitflags::bitflags;
use std::ptr;
use std::{ffi::CStr, os::windows::ffi::OsStrExt};
use std::{
    ffi::{OsStr, OsString},
    os::windows::prelude::OsStringExt,
};
use winapi::um::minwinbase::WIN32_FIND_DATAW;
use winapi::um::winnt::{FILE_ATTRIBUTE_DIRECTORY, HANDLE};
use winapi::{shared::minwindef::DWORD, um::handleapi::INVALID_HANDLE_VALUE};
use winapi::{
    shared::minwindef::FILETIME,
    um::fileapi::{FindClose, FindFirstFileW, FindNextFileW},
};

bitflags! {
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

impl Into<FileTime> for FILETIME {
    fn into(self) -> FileTime {
        FileTime {
            low: self.dwLowDateTime,
            high: self.dwHighDateTime,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DirEntry {
    pub file_name: OsString,
    pub directory: bool,
    pub creation_time: FileTime,
    pub last_access: FileTime,
    pub last_write: FileTime,
    pub file_attributes: FileAttributes,
}

//TODO: Depth + Recursion.
pub fn walkdir<S: AsRef<str>>(path: S) -> Result<Vec<DirEntry>, ()> {
    let search_pattern_wide: Vec<u16> = OsStr::new(path.as_ref())
        .encode_wide()
        .chain(Some(b'\\' as u16).into_iter())
        .chain(Some(b'*' as u16).into_iter())
        .chain(Some(0).into_iter())
        .collect();

    let mut find_data: WIN32_FIND_DATAW = unsafe { std::mem::zeroed() };

    #[rustfmt::skip]
    let search_handle: HANDLE = unsafe { FindFirstFileW(search_pattern_wide.as_ptr(), &mut find_data) };

    let mut files = Vec::new();

    if search_handle != ptr::null_mut() && search_handle != INVALID_HANDLE_VALUE {
        loop {
            let nul_range_end = find_data
                .cFileName
                .iter()
                .position(|&c| c == b'\0' as u16)
                .unwrap_or(find_data.cFileName.len());
            let file_name = OsString::from_wide(&find_data.cFileName[..nul_range_end]);
            let is_directory = (find_data.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0;

            let creation_time = find_data.ftCreationTime;
            let last_access = find_data.ftLastAccessTime;
            let last_write = find_data.ftLastWriteTime;

            let file_attributes = FileAttributes::from_bits_truncate(find_data.dwFileAttributes);
            dbg!(file_attributes);

            let file_size = files.push(DirEntry {
                file_name,
                directory: is_directory,
                creation_time: creation_time.into(),
                last_access: last_access.into(),
                last_write: last_write.into(),
                file_attributes,
            });

            find_data = unsafe { std::mem::zeroed() };
            if unsafe { FindNextFileW(search_handle, &mut find_data) == 0 } {
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

fn main() {
    for files in walkdir("D:\\Desktop").unwrap() {
        // dbg!(files);
    }
}
