use base_db::{FileLoader, FilePosition};
use hir::Semantics;
use hir_def::DefDatabase;
use lsp_types::LocationLink;
use vfs::FileId;

use crate::RootDatabase;

/// Function to find a sub-node by ID.
fn find_sub_node_by_id(node: tree_sitter::Node, target_id: usize) -> Option<tree_sitter::Node> {
    if node.id() == target_id {
        return Some(node);
    }

    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        if let Some(found) = find_sub_node_by_id(child, target_id) {
            return Some(found);
        }
    }

    None
}

fn position_from_ts_point(ts_point: tree_sitter::Point) -> lsp_types::Position {
    lsp_types::Position {
        line: ts_point.row as u32,
        character: ts_point.column as u32,
    }
}

fn range_from_ts_range(ts_range: tree_sitter::Range) -> lsp_types::Range {
    lsp_types::Range {
        start: position_from_ts_point(ts_range.start_point),
        end: position_from_ts_point(ts_range.end_point),
    }
}

pub struct NavigationTarget {
    pub file_id: FileId,
    pub origin_selection_range: lsp_types::Range,
    pub target_range: lsp_types::Range,
    pub target_selection_range: lsp_types::Range,
}

pub(crate) fn goto_definition(
    db: &RootDatabase,
    pos: FilePosition,
) -> Option<Vec<NavigationTarget>> {
    log::info!("Going to def.");
    let sema = &Semantics::new(db);
    let file = sema.parse(pos.file_id);
    let node = file.node_from_pos(pos.position)?;
    let node_id = sema.find_def(pos.file_id, node)?;
    let root_node = file.tree().root_node();
    let def_node = find_sub_node_by_id(root_node, node_id)?;
    log::info!(
        "{:?}",
        node.utf8_text(db.file_text(pos.file_id).as_ref().as_bytes())
    );
    log::info!("{:?}", db.file_item_tree(pos.file_id));
    let mut name_range = def_node.range();
    if let Some(name_node) = def_node.child_by_field_name("name") {
        name_range = name_node.range();
    }
    Some(vec![NavigationTarget {
        file_id: pos.file_id,
        origin_selection_range: range_from_ts_range(node.range()),
        target_range: range_from_ts_range(def_node.range()),
        target_selection_range: range_from_ts_range(name_range),
    }])
}

// We have an item tree and queries for each item that compute their data.
