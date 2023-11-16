fn main() {
    divan::main();
}

const PATH: &str = "C:\\Windows\\System32";
const DEPTH: usize = 1;

#[divan::bench]
fn winwalk() {
    use winwalk::*;
    let results: Vec<Result<DirEntry, Error>> = walkdir(divan::black_box(PATH), DEPTH);
    assert!(!results.is_empty());
}

#[divan::bench]
fn walkdir() {
    use walkdir::{DirEntry, Error, WalkDir};
    let results: Vec<Result<DirEntry, Error>> = WalkDir::new(divan::black_box(PATH))
        .max_depth(DEPTH)
        .into_iter()
        .collect();
    assert!(!results.is_empty());
}
