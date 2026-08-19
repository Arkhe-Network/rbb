use criterion::{criterion_group, criterion_main, Criterion};

pub fn dummy_bench(_c: &mut Criterion) {}

criterion_group!(benches, dummy_bench);
criterion_main!(benches);
