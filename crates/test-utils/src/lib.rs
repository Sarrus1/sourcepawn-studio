use std::path::{Path, PathBuf};

/// Get the workspace root.
pub fn project_root() -> PathBuf {
    let dir = env!("CARGO_MANIFEST_DIR");
    let res = PathBuf::from(dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();
    res
}

/// Checks that the `file` has the specified `contents`. If that is not the
/// case, updates the file and then fails the test.
#[track_caller]
pub fn ensure_file_contents(file: &Path, contents: &str) {
    if let Err(()) = try_ensure_file_contents(file, contents) {
        panic!("Some files were not up-to-date");
    }
}

/// Checks that the `file` has the specified `contents`. If that is not the
/// case, updates the file and return an Error.
#[allow(clippy::result_unit_err)]
pub fn try_ensure_file_contents(file: &Path, contents: &str) -> Result<(), ()> {
    match std::fs::read_to_string(file) {
        Ok(old_contents) if normalize_newlines(&old_contents) == normalize_newlines(contents) => {
            return Ok(());
        }
        _ => (),
    }
    let display_path = file.strip_prefix(project_root()).unwrap_or(file);
    eprintln!(
        "\n\x1b[31;1merror\x1b[0m: {} was not up-to-date, updating\n",
        display_path.display()
    );
    if let Some(parent) = file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(file, contents).unwrap();
    Err(())
}

fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n")
}
