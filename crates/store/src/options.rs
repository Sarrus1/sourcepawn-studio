use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct Options {
    pub includes_directories: Vec<PathBuf>,
    pub spcomp_path: PathBuf,
    pub linter_arguments: Vec<String>,
    pub disable_syntax_linter: bool,
}

impl Options {
    /// Return all possible include folder paths.
    ///
    /// # Arguments
    /// * `main_path` - [Path](PathBuf) of the main file.
    pub fn get_all_possible_include_folders(&self, main_path: PathBuf) -> Vec<PathBuf> {
        let mut res: Vec<PathBuf> = vec![];
        for path in self.includes_directories.iter() {
            if path.exists() {
                res.push(path.clone());
            }
        }

        if let Some(scripting_folder) = main_path.parent() {
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
