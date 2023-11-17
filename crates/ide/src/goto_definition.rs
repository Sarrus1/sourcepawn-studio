use base_db::{FileLoader, FilePosition};
use hir::Semantics;
use hir_def::DefDatabase;
use lsp_types::LocationLink;

use crate::RootDatabase;

pub(crate) fn goto_definition(db: &RootDatabase, pos: FilePosition) -> Option<Vec<LocationLink>> {
    log::info!("Going to def.");
    let sema = &Semantics::new(db);
    let file = sema.parse(pos.file_id);
    let node = file.node_from_pos(pos.position)?;
    let node_id = sema.find_def(pos.file_id, node)?;
    let root_node = file.tree().root_node();
    let mut cursor = file.tree().root_node().walk();
    for child in root_node.children(&mut cursor) {
        log::info!("{:?}", node_id);
        if child.id() == node_id {
            log::info!("FOUND");
        }
    }
    log::info!(
        "{:?}",
        node.utf8_text(db.file_text(pos.file_id).as_ref().as_bytes())
    );
    log::info!("{:?}", db.file_item_tree(pos.file_id));
    None
}

// We have an item tree and queries for each item that compute their data.
