use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct Options {
    pub includes_directories: Vec<PathBuf>,
    pub main_path: PathBuf,
}

impl Options {
    /// Return all possible include folder paths.
    pub fn get_all_possible_include_folder(&self) -> Vec<PathBuf> {
        let mut res: Vec<PathBuf> = vec![];
        for path in self.includes_directories.iter() {
            if path.exists() {
                res.push(path.clone());
            }
        }

        if let Some(scripting_folder) = self.main_path.parent() {
            if scripting_folder.exists() {
                res.push(scripting_folder.to_path_buf());
            }
            let include_folder = scripting_folder.join("include");
            if include_folder.exists() {
                res.push(include_folder);
            }
        }

        res
    }
}
