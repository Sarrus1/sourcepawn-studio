use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::{
    fs::File,
    io::{self, Read},
};
use tempfile::tempdir;
use walkdir::WalkDir;
use zip::ZipArchive;

use sourcepawn_lsp::fixture::complete;

fn download_file(url: &str) -> Vec<u8> {
    let response = minreq::get(url).send().unwrap();
    response.as_bytes().to_vec()
}

fn unzip_file(zip_data: &[u8], destination: &str) -> Result<(), io::Error> {
    let reader = io::Cursor::new(zip_data);
    let mut archive = ZipArchive::new(reader)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let dest_path = format!("{}/{}", destination, file.name());

        if file.is_dir() {
            std::fs::create_dir_all(&dest_path)?;
        } else {
            if let Some(parent_dir) = std::path::Path::new(&dest_path).parent() {
                std::fs::create_dir_all(parent_dir)?;
            }
            let mut dest_file = File::create(&dest_path)?;
            io::copy(&mut file, &mut dest_file)?;
        }
    }

    Ok(())
}

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
                    let path = entry.path().to_string_lossy();
                    let substring = "/addons/sourcemod/scripting/";
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
    let zip_data = download_file("https://github.com/surftimer/SurfTimer/archive/384784b5d21163553e07e00a2313520426cb195f.zip");
    unzip_file(&zip_data, tmp_dir_path.to_str().unwrap()).unwrap();
    let mut fixture = create_fixture(tmp_dir_path.to_str().unwrap()).unwrap();
    fixture.push_str("\n\n|\n^");

    c.bench_function("surftimer_end2end", |b| {
        b.iter(|| {
            let _res = black_box(complete(&fixture));
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
