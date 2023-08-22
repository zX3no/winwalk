use criterion::*;
use std::{
    ffi::OsStr,
    os::windows::{prelude::OsStrExt, raw::HANDLE},
    path::Path,
};
use winapi::um::{
    fileapi::{FindClose, FindFirstFileW, FindNextFileW},
    handleapi::INVALID_HANDLE_VALUE,
    minwinbase::WIN32_FIND_DATAW,
    winnt::FILE_ATTRIBUTE_DIRECTORY,
};
use winwalk::*;

fn baseline<S: AsRef<Path>>(path: S, depth: usize) -> Vec<WIN32_FIND_DATAW> {
    unsafe {
        let path = path.as_ref();
        let search_pattern_wide: Vec<u16> = OsStr::new(path)
            .encode_wide()
            .chain(Some(b'\\' as u16).into_iter())
            .chain(Some(b'*' as u16).into_iter())
            .chain(Some(0).into_iter())
            .collect();

        let mut fd: WIN32_FIND_DATAW = std::mem::zeroed();
        let search_handle: HANDLE = FindFirstFileW(search_pattern_wide.as_ptr(), &mut fd);
        let mut files = Vec::new();

        if search_handle != std::ptr::null_mut() && search_handle != INVALID_HANDLE_VALUE {
            loop {
                let is_folder = (fd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0;

                if is_folder {
                    if depth != 0 {
                        if depth - 1 != 0 {
                            files.extend(baseline(&path, depth - 1));
                        }
                    } else {
                        files.extend(baseline(&path, 0));
                    }
                }

                files.push(fd);

                fd = std::mem::zeroed();

                if FindNextFileW(search_handle, &mut fd) == 0 {
                    break;
                }
            }

            FindClose(search_handle);

            files
        } else {
            files
        }
    }
}

const PATH: &str = "C:\\Windows\\System32";
const DEPTH: usize = 1;

fn bench_walks(c: &mut Criterion) {
    let mut group = c.benchmark_group("Walks");

    group.bench_function("Baseline", |b| {
        b.iter(|| {
            //
            baseline(PATH, DEPTH);
        });
    });

    group.bench_function("Walkdir", |b| {
        b.iter(|| {
            //
            walkdir(PATH, DEPTH);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_walks);
criterion_main!(benches);
