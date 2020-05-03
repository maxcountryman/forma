use criterion::{criterion_group, criterion_main, Criterion};
use formation::prettify;

fn basic_formatting_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Formation Formatting");
    let string = "SELECT * FROM bob WHERE 1 = 1";
    group.bench_function("forma::prettify", |b| {
        b.iter(|| prettify(string.to_string(), 80));
    });
}

criterion_group!(benches, basic_formatting_benchmark);
criterion_main!(benches);
