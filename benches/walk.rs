use criterion::*;
use std::{ffi::c_void, path::Path};
use winwalk::*;

fn baseline<S: AsRef<Path>>(path: S, depth: usize) -> Vec<FindDataA> {
    unsafe {
        let path = path.as_ref();
        let search_pattern = [path.as_os_str().as_encoded_bytes(), &[b'\\', b'*', 0]].concat();

        let mut fd: FindDataA = std::mem::zeroed();
        let search_handle: *mut c_void =
            FindFirstFileA(search_pattern.as_ptr() as *mut i8, &mut fd);
        let mut files = Vec::new();

        if search_handle != std::ptr::null_mut() && search_handle != INVALID_HANDLE_VALUE {
            loop {
                let is_folder = (fd.file_attributes & FILE_ATTRIBUTE_DIRECTORY) != 0;

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

                if !FindNextFileA(search_handle, &mut fd) {
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
