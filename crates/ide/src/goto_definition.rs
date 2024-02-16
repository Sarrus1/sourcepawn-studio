use base_db::FilePosition;
use hir::{DefResolution, HasSource, Semantics};

use syntax::utils::{lsp_position_to_ts_point, ts_range_to_lsp_range};
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
    let preprocessing_results = sema.preprocess_file(pos.file_id);
    let (def, range) =
        if let Some(offsets) = preprocessing_results.offsets().get(&pos.position.line) {
            let offset = offsets
                .iter()
                .find(|offset| offset.contains(pos.position))?;
            (
                sema.find_macro_def(offset.file_id, offset.idx)
                    .map(DefResolution::from)?,
                offset.range,
            )
        } else {
            let tree = sema.parse(pos.file_id);
            let root_node = tree.root_node();
            let node = root_node.descendant_for_point_range(
                lsp_position_to_ts_point(&pos.position),
                lsp_position_to_ts_point(&pos.position),
            )?;
            (
                sema.find_def(pos.file_id, &node)?,
                ts_range_to_lsp_range(&node.range()),
            )
        };

    let file_id = def.file_id(db);
    let source_tree = sema.parse(file_id);
    let def_node = def.source(db, &source_tree)?.value;

    let mut name_range = def_node.range();
    if let Some(name_node) = def_node.child_by_field_name("name") {
        name_range = name_node.range();
    }

    Some(vec![NavigationTarget {
        file_id,
        origin_selection_range: range,
        target_range: ts_range_to_lsp_range(&def_node.range()),
        target_selection_range: ts_range_to_lsp_range(&name_range),
    }])
}
