const PATH: &str = "C:\\Windows\\System32";
const DEPTH: usize = 1;

fn main() {
    divan::main();
}

#[divan::bench]
fn local() {
    let results = winwalk::walkdir(divan::black_box(PATH), DEPTH);
    assert!(!results.is_empty());
}

#[divan::bench]
fn walk() {
    let results: Vec<_> = walkdir::WalkDir::new(divan::black_box(PATH))
        .max_depth(DEPTH)
        .into_iter()
        .flatten()
        .collect();
    assert!(!results.is_empty());
}
