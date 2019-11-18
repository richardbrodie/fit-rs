#[macro_use]
extern crate criterion;
extern crate fit;

use criterion::BenchmarkId;
use criterion::Criterion;
use std::path::PathBuf;

fn bench_read_file(filepath: &PathBuf) {
    let f = fit::Fit::new(&filepath);
    for m in f {
        m.kind;
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let s = "data/garmin_1000.fit";
    let filepath = PathBuf::from(s);
    c.bench_with_input(
        BenchmarkId::new("bench_read_file", s), &filepath, |b, filepath| {
            b.iter(|| bench_read_file(&filepath));
        }
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
