use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::{
    env,
    fs::File,
    io::{self, Read},
};
use tempfile::tempdir;
use walkdir::WalkDir;

use sourcepawn_lsp::fixture::{self, complete};

fn create_fixture(dir_path: &str) -> Result<String, io::Error> {
    let mut result = String::new();

    for entry in WalkDir::new(dir_path).follow_links(true) {
        let entry = entry?;
        if entry.file_type().is_file() {
            if let Some(extension) = entry.path().extension() {
                if extension == "sp" || extension == "inc" {
                    let mut file = File::open(entry.path())?;
                    let mut file_content = String::new();
                    file.read_to_string(&mut file_content)?;
                    let substring = "/scripting/";
                    let path = entry.path().to_string_lossy();
                    let cropped_path = path
                        .split_at(path.find(substring).unwrap() + substring.len())
                        .1;
                    result.push_str(&format!(
                        r#"%! {}
{}
"#,
                        cropped_path, file_content
                    ));
                }
            }
        }
    }

    Ok(result)
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let tmp_dir = tempdir().unwrap();
    let tmp_dir_path = tmp_dir.path().canonicalize().unwrap();
    let current_dir = env::current_dir().unwrap();
    let surtimer_path = current_dir.join("test_data/surftimer.zip");
    fixture::unzip_file(&surtimer_path, &tmp_dir_path).unwrap();

    let mut fixture = create_fixture(tmp_dir_path.to_str().unwrap()).unwrap();
    fixture.push_str("\n\n|\n^");

    c.bench_function("surftimer_end2end", |b| {
        b.iter(|| {
            let _res = black_box(complete(&fixture, None));
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
