use base_db::{FileLoader, FilePosition};
use hir::Semantics;
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
    let file = sema.parse(pos.file_id);
    let root_node = file.root_node();
    let node = root_node.descendant_for_point_range(
        lsp_position_to_ts_point(&pos.position),
        lsp_position_to_ts_point(&pos.position),
    )?;
    let def_ptr = sema.find_def(pos.file_id, &node)?;
    let def_node = def_ptr.to_node(&file);
    log::info!(
        "{:?}",
        node.utf8_text(db.file_text(pos.file_id).as_ref().as_bytes())
    );
    log::info!("{:?}", db.file_item_tree(pos.file_id));
    let mut name_range = def_node.range();

    match TSKind::from(def_node) {
        TSKind::sym_function_declaration => {
            if let Some(name_node) = def_node.child_by_field_name("name") {
                name_range = name_node.range();
            }
        }
        TSKind::sym_variable_declaration => {
            if let Some(name_node) = def_node.child_by_field_name("name") {
                name_range = name_node.range();
            }
        }
        _ => todo!(),
    }
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
