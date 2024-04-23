use std::hash::Hash;

use base_db::FilePosition;
use fxhash::FxHashMap;
use hir::{DefResolution, HasSource, Semantics};

use preprocessor::Offset;
use smol_str::{SmolStr, ToSmolStr};
use syntax::{
    utils::{lsp_position_to_ts_point, ts_range_to_lsp_range},
    TSKind,
};
use vfs::FileId;

use crate::{s_range_to_u_range, u_pos_to_s_pos, RangeInfo, RootDatabase};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NavigationTarget {
    pub name: SmolStr,
    pub file_id: FileId,
    pub full_range: lsp_types::Range,
    pub focus_range: Option<lsp_types::Range>,
}

impl Hash for NavigationTarget {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.file_id.hash(state);
        self.full_range.start.line.hash(state);
        self.full_range.start.character.hash(state);
        self.full_range.end.line.hash(state);
        self.full_range.end.character.hash(state);
        if let Some(focus_range) = &self.focus_range {
            focus_range.start.line.hash(state);
            focus_range.start.character.hash(state);
            focus_range.end.line.hash(state);
            focus_range.end.character.hash(state);
        }
    }
}

impl NavigationTarget {
    pub fn focus_or_full_range(&self) -> lsp_types::Range {
        self.focus_range.unwrap_or(self.full_range)
    }
}

pub(crate) fn goto_definition(
    db: &RootDatabase,
    mut pos: FilePosition,
) -> Option<RangeInfo<Vec<NavigationTarget>>> {
    let sema = &Semantics::new(db);
    let preprocessing_results = sema.preprocess_file(pos.file_id);
    let offsets = preprocessing_results.offsets();
    let tree = sema.parse(pos.file_id);
    let root_node = tree.root_node();

    if let Some((offset, def)) = find_macro_def(offsets, &pos.position, sema) {
        let file_id = def.file_id(sema.db);
        let u_range = offset.range;
        let source_tree = sema.parse(file_id);
        let name = def.name(db).map(|it| it.to_smolstr()).unwrap_or_default();
        let def_node = def.source(sema.db, &source_tree)?.value;
        let name_range = find_inner_name_range(&def_node);
        let navs = vec![NavigationTarget {
            name,
            file_id,
            full_range: ts_range_to_lsp_range(&def_node.range()),
            focus_range: name_range.into(),
        }];

        return RangeInfo::new(u_range, navs).into();
    }

    let source_u_range =
        u_pos_to_s_pos(preprocessing_results.args_map(), offsets, &mut pos.position);

    let node = root_node.descendant_for_point_range(
        lsp_position_to_ts_point(&pos.position),
        lsp_position_to_ts_point(&pos.position),
    )?;
    let def = sema.find_def(pos.file_id, &node)?;
    let u_range = match source_u_range {
        Some(u_range) => u_range,
        None => s_range_to_u_range(offsets, ts_range_to_lsp_range(&node.range())),
    };

    let file_id = def.file_id(db);
    let source_tree = sema.parse(file_id);
    let name = def.name(db).map(|it| it.to_smolstr()).unwrap_or_default();
    let def_node = def.source(db, &source_tree)?.value;

    let name_range = find_inner_name_range(&def_node);

    let target_preprocessing_results = sema.preprocess_file(file_id);
    let target_offsets = target_preprocessing_results.offsets();
    let navs = vec![NavigationTarget {
        name,
        file_id,
        full_range: s_range_to_u_range(target_offsets, ts_range_to_lsp_range(&def_node.range())),
        focus_range: s_range_to_u_range(target_offsets, name_range).into(),
    }];

    RangeInfo::new(u_range, navs).into()
}

/// Find the range of the inner name node of a definition node if there is one.
/// Otherwise, return the range of the definition node.
pub fn find_inner_name_range(node: &tree_sitter::Node) -> lsp_types::Range {
    let name_range = match TSKind::from(node) {
        TSKind::methodmap_property_native | TSKind::methodmap_property_method => {
            node.children(&mut node.walk()).find_map(|child| {
                if matches!(
                    TSKind::from(child),
                    TSKind::methodmap_property_getter | TSKind::methodmap_property_setter
                ) {
                    Some(child.child_by_field_name("name")?.range())
                } else {
                    None
                }
            })
        }
        _ => node
            .child_by_field_name("name")
            .map(|name_node| name_node.range()),
    }
    .unwrap_or_else(|| node.range());

    ts_range_to_lsp_range(&name_range)
}

/// Try to find the definition of a macro at the given position.
pub fn find_macro_def(
    offsets: &FxHashMap<u32, Vec<Offset>>,
    pos: &lsp_types::Position,
    sema: &Semantics<RootDatabase>,
) -> Option<(Offset, DefResolution)> {
    let offset = offsets
        .get(&pos.line)
        .and_then(|offsets| offsets.iter().find(|offset| offset.contains(*pos)))?;
    (
        offset.to_owned(),
        sema.find_macro_def(offset.file_id, offset.idx)
            .map(DefResolution::from)?,
    )
        .into()
}
