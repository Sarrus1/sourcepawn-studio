use std::hash::Hash;

use base_db::FilePosition;
use hir::{HasSource, Semantics};

use line_index::TextRange;
use smol_str::{SmolStr, ToSmolStr};
use syntax::{utils::ts_range_to_text_range, TSKind};
use vfs::FileId;

use crate::{RangeInfo, RootDatabase};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NavigationTarget {
    pub name: SmolStr,
    pub file_id: FileId,
    pub full_range: TextRange,
    pub focus_range: Option<TextRange>,
}

impl NavigationTarget {
    pub fn focus_or_full_range(&self) -> TextRange {
        self.focus_range.unwrap_or(self.full_range)
    }
}

pub(crate) fn goto_definition(
    db: &RootDatabase,
    pos: FilePosition,
) -> Option<RangeInfo<Vec<NavigationTarget>>> {
    let sema = &Semantics::new(db);
    let preprocessing_results = sema.preprocess_file(pos.file_id);
    let tree = sema.parse(pos.file_id);
    let root_node = tree.root_node();

    if let Some((offset, def)) = sema.find_macro_def(&pos) {
        let file_id = def.file_id(sema.db);
        let u_range = offset.name_range();
        let source_tree = sema.parse(file_id);
        let name = def.name(db).map(|it| it.to_smolstr()).unwrap_or_default();
        let def_node = def.source(sema.db, &source_tree)?.value;
        let name_range = find_inner_name_range(&def_node);
        let navs = vec![NavigationTarget {
            name,
            file_id,
            full_range: ts_range_to_text_range(&def_node.range()),
            focus_range: name_range.into(),
        }];

        return RangeInfo::new(u_range, navs).into();
    }

    let offset: u32 = preprocessing_results
        .source_map()
        .closest_s_position_always(pos.offset)
        .into();

    let node = root_node.descendant_for_byte_range(offset as usize, offset as usize)?;
    let def = sema.find_def(pos.file_id, &node)?;
    let ts_range = ts_range_to_text_range(&node.range());
    let u_range = preprocessing_results
        .source_map()
        .closest_u_range_always(ts_range);

    let file_id = def.file_id(db);
    let source_tree = sema.parse(file_id);
    let name = def.name(db).map(|it| it.to_smolstr()).unwrap_or_default();
    let def_node = def.source(db, &source_tree)?.value;

    let name_range = find_inner_name_range(&def_node);

    let target_preprocessing_results = sema.preprocess_file(file_id);
    let navs = vec![NavigationTarget {
        name,
        file_id,
        full_range: target_preprocessing_results
            .source_map()
            .closest_u_range_always(ts_range_to_text_range(&def_node.range())),
        focus_range: target_preprocessing_results
            .source_map()
            .closest_u_range_always(name_range)
            .into(),
    }];

    RangeInfo::new(u_range, navs).into()
}

/// Find the range of the inner name node of a definition node if there is one.
/// Otherwise, return the range of the definition node.
pub fn find_inner_name_range(node: &tree_sitter::Node) -> TextRange {
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

    ts_range_to_text_range(&name_range)
}
