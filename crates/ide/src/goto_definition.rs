use base_db::{FileLoader, FilePosition, SourceDatabase};
use hir_def::DefDatabase;
use lsp_types::LocationLink;

use crate::RootDatabase;

pub(crate) fn goto_definition(db: &RootDatabase, pos: FilePosition) -> Option<Vec<LocationLink>> {
    log::info!("Going to def.");
    let file = db.parse(pos.file_id);
    let node = file.node_from_pos(pos.position)?;
    log::info!(
        "{:?}",
        node.utf8_text(db.file_text(pos.file_id).as_ref().as_bytes())
    );
    log::info!("{:?}", db.file_item_tree(pos.file_id));
    None
}

// We have an item tree and queries for each item that compute their data.
