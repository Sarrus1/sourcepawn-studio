use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fxhash::FxHashMap;
use preprocessor::{Macro, SourcepawnPreprocessor};
use vfs::FileId;

fn extend_macros(
    _macros: &mut FxHashMap<String, Macro>,
    mut _path: String,
    _file_id: FileId,
    _quoted: bool,
) -> anyhow::Result<()> {
    Ok(())
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let response = minreq::get("https://raw.githubusercontent.com/surftimer/SurfTimer/32d9777f3fb2ba1b2b5930493cf7d0d01dc3e40d/addons/sourcemod/scripting/surftimer/sql.sp")
        .with_body("Hello, world!")
        .send().unwrap();
    let input = response.as_str().unwrap();

    c.bench_function("surftimer_sql", |b| {
        b.iter(|| {
            let _res = black_box(
                SourcepawnPreprocessor::new(FileId::from(0), input)
                    .preprocess_input(&mut extend_macros)
                    .unwrap()
                    .preprocessed_text(),
            );
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
