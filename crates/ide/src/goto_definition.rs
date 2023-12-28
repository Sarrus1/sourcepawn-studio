use base_db::{FileLoader, FilePosition};
use hir::{HasSource, Semantics};
use hir_def::DefDatabase;
use syntax::{
    utils::{lsp_position_to_ts_point, ts_range_to_lsp_range},
    TSKind,
};
use vfs::FileId;

use crate::RootDatabase;

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
    let tree = sema.parse(pos.file_id);
    let root_node = tree.root_node();
    let node = root_node.descendant_for_point_range(
        lsp_position_to_ts_point(&pos.position),
        lsp_position_to_ts_point(&pos.position),
    )?;
    let def_node = sema.find_def(pos.file_id, &node)?.source(db, &tree)?.value;

    let mut name_range = def_node.range();
    if let Some(name_node) = def_node.child_by_field_name("name") {
        name_range = name_node.range();
    }
    Some(vec![NavigationTarget {
        file_id: pos.file_id,
        origin_selection_range: ts_range_to_lsp_range(&node.range()),
        target_range: ts_range_to_lsp_range(&def_node.range()),
        target_selection_range: ts_range_to_lsp_range(&name_range),
    }])
}
