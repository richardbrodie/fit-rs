#[macro_use]
extern crate criterion;
extern crate fit;

use criterion::Criterion;
use std::path::PathBuf;

fn read_file() {
    let filepath = PathBuf::from("data/garmin_1000.fit");
    fit::run(&filepath);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("read_file", |b| b.iter(|| read_file()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
