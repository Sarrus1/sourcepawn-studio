use std::sync::Arc;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fxhash::FxHashMap;
use lsp_types::Url;
use preprocessor::{Macro, SourcepawnPreprocessor};

fn extend_macros(
    _macros: &mut FxHashMap<String, Macro>,
    mut _path: String,
    _document_uri: &Url,
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
                SourcepawnPreprocessor::new(
                    Arc::new(Url::parse("https://example.net").unwrap()),
                    input,
                )
                .preprocess_input(&mut extend_macros)
                .unwrap_or_else(|err| {
                    eprintln!("{:?}", err);
                    "".to_string()
                }),
            );
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
