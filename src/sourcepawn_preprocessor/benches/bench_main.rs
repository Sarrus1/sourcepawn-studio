use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sourcepawn_preprocessor::SourcepawnPreprocessor;

pub fn criterion_benchmark(c: &mut Criterion) {
    let response = minreq::get("https://raw.githubusercontent.com/surftimer/SurfTimer/32d9777f3fb2ba1b2b5930493cf7d0d01dc3e40d/addons/sourcemod/scripting/surftimer/sql.sp")
        .send().unwrap();
    let input = response.as_str().unwrap();
    c.bench_function("surftimer_sql", |b| {
        b.iter(|| {
            black_box(SourcepawnPreprocessor::new(input).preprocess_input());
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
