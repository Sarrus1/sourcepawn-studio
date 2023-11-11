//! Defines a unit of change that can applied to the database to get the next
//! state. Changes are transactional.

use std::{fmt, sync::Arc};

use salsa::Durability;
use vfs::FileId;

use crate::SourceDatabaseExt;

/// Encapsulate a bunch of raw `.set` calls on the database.
#[derive(Default)]
pub struct Change {
    pub files_changed: Vec<(FileId, Option<Arc<str>>)>,
}

impl fmt::Debug for Change {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = fmt.debug_struct("Change");
        if !self.files_changed.is_empty() {
            d.field("files_changed", &self.files_changed.len());
        }
        d.finish()
    }
}

impl Change {
    pub fn new() -> Self {
        Change::default()
    }

    pub fn change_file(&mut self, file_id: FileId, new_text: Option<Arc<str>>) {
        self.files_changed.push((file_id, new_text))
    }

    pub fn apply(self, db: &mut dyn SourceDatabaseExt) {
        for (file_id, text) in self.files_changed {
            let text = text.unwrap_or_else(|| Arc::from(""));
            db.set_file_text(file_id, text)
        }
    }
}
