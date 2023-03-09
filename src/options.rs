use std::path::PathBuf;

use lsp_types::Url;
use serde::{Deserialize, Serialize};

use crate::utils::normalize_uri;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct Options {
    pub includes_directories: Vec<PathBuf>,
    pub main_path: PathBuf,
    pub spcomp_path: PathBuf,
}

impl Options {
    /// Return all possible include folder paths.
    pub fn get_all_possible_include_folders(&self) -> Vec<PathBuf> {
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

    /// Return the [uri](lsp_types::Url) main path. [None] if it is empty. [Err] otherwise.
    pub fn get_main_path_uri(&self) -> anyhow::Result<Option<Url>> {
        if self.main_path.to_str().unwrap().is_empty() {
            return Ok(None);
        }
        if !self.main_path.exists() {
            return Err(anyhow::anyhow!("Main path does not exist."));
        }
        let main_uri = Url::from_file_path(&self.main_path);
        if let Ok(mut main_uri) = main_uri {
            normalize_uri(&mut main_uri);
            return Ok(Some(main_uri));
        }

        Err(anyhow::anyhow!("Main path does not exist."))
    }

    /// Returns true if the given path is a parent or one of the IncludeDirectories.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to check against.
    pub fn is_parent_of_include_dir(&self, path: &PathBuf) -> bool {
        for include_dir in self.includes_directories.iter() {
            if include_dir.starts_with(path) {
                return true;
            }
        }

        false
    }
}
