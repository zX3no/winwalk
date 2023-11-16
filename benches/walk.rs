fn main() {
    divan::main();
}

const PATH: &str = "C:\\Windows\\System32";
const DEPTH: usize = 1;

#[divan::bench]
fn winwalk() {
    let results = winwalk::walkdir(divan::black_box(PATH), DEPTH);
    assert!(!results.is_empty());
}

#[divan::bench]
fn walkdir() {
    let results: Vec<_> = walkdir::WalkDir::new(divan::black_box(PATH))
        .max_depth(DEPTH)
        .into_iter()
        .flatten()
        .collect();
    assert!(!results.is_empty());
}
