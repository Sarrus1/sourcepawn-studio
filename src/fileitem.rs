use std::collections::{HashMap, HashSet};

use derive_new::new;

use crate::spitem::SPItem;

#[derive(Debug, Default, new)]
pub struct FileItem {
    pub uri: String,
    pub text: String,
    pub sp_items: Vec<SPItem>,
    pub includes: HashSet<String>,
}

impl FileItem {
    fn resolve_import(
        include_text: &String,
        documents: &HashSet<String>,
        file_path: &String,
    ) -> String {
        "".to_string()
    }
}
