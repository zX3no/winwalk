use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::Path;
use winwalk::*;

fn walk(c: &mut Criterion) {
    c.bench_function("Function B", |b| {
        b.iter(|| walkdir(black_box(Path::new("C:\\Windows\\System32")), 1));
    });
}

criterion_group!(benches, walk);
criterion_main!(benches);
