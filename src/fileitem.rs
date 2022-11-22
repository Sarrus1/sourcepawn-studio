use derive_new::new;

use crate::spitem::SPItem;

#[derive(Debug, Default, new)]
pub struct FileItem {
    pub uri: String,
    pub text: String,
    pub sp_items: Vec<SPItem>,
}
