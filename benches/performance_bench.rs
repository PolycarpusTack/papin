use criterion::{criterion_group, criterion_main, Criterion};

fn bench_dummy(c: &mut Criterion) {
    c.bench_function("dummy", |b| b.iter(|| 1 + 1));
}

criterion_group!(benches, bench_dummy);
criterion_main!(benches);
