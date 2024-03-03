use base_db::FilePosition;
use fxhash::FxHashMap;
use hir::{DefResolution, HasSource, Semantics};

use preprocessor::Offset;
use syntax::{
    range_contains_pos,
    utils::{lsp_position_to_ts_point, ts_range_to_lsp_range},
    TSKind,
};
use vfs::FileId;

use crate::{RangeInfo, RootDatabase};

pub struct NavigationTarget {
    pub file_id: FileId,
    pub full_range: lsp_types::Range,
    pub focus_range: Option<lsp_types::Range>,
}

pub(crate) fn goto_definition(
    db: &RootDatabase,
    mut pos: FilePosition,
) -> Option<RangeInfo<Vec<NavigationTarget>>> {
    let sema = &Semantics::new(db);
    let preprocessing_results = sema.preprocess_file(pos.file_id);
    let offsets = preprocessing_results.offsets();
    if let Some(offset) = offsets
        .get(&pos.position.line)
        .and_then(|offsets| offsets.iter().find(|offset| offset.contains(pos.position)))
    {
        let def = sema
            .find_macro_def(offset.file_id, offset.idx)
            .map(DefResolution::from)?;
        let file_id = def.file_id(db);
        let u_range = offset.range;
        let source_tree = sema.parse(file_id);
        let def_node = def.source(db, &source_tree)?.value;
        let mut name_range = def_node.range();
        if let Some(name_node) = def_node.child_by_field_name("name") {
            name_range = name_node.range();
        }
        let navs = vec![NavigationTarget {
            file_id,
            full_range: ts_range_to_lsp_range(&def_node.range()),
            focus_range: ts_range_to_lsp_range(&name_range).into(),
        }];

        return RangeInfo::new(u_range, navs).into();
    }

    let diff = if let Some(diff) = preprocessing_results
        .args_map()
        .get(&pos.position.line)
        .and_then(|mapped_ranges| {
            mapped_ranges
                .iter()
                .find(|(range, _)| range_contains_pos(range, &pos.position))
        })
        .map(|(range, mapped_range)| {
            mapped_range.start.character as i32 - range.start.character as i32
        }) {
        pos.position.character = pos.position.character.saturating_add_signed(diff);
        diff.into()
    } else {
        None
    };

    let tree = sema.parse(pos.file_id);
    let root_node = tree.root_node();
    let s_pos = u_pos_to_s_pos(offsets, pos.position);
    let node = root_node.descendant_for_point_range(
        lsp_position_to_ts_point(&s_pos),
        lsp_position_to_ts_point(&s_pos),
    )?;

    let def = sema.find_def(pos.file_id, &node)?;

    let mut u_range = ts_range_to_lsp_range(&node.range());
    if let Some(diff) = diff {
        u_range.start.character = u_range.start.character.saturating_add_signed(-diff);
        u_range.end.character = u_range.end.character.saturating_add_signed(-diff);
    } else {
        u_range = s_range_to_u_range(offsets, u_range);
    }

    let file_id = def.file_id(db);
    let source_tree = sema.parse(file_id);
    let def_node = def.source(db, &source_tree)?.value;

    let mut name_range = def_node.range();
    let inner_name_range = match TSKind::from(def_node) {
        TSKind::methodmap_property_native | TSKind::methodmap_property_method => {
            def_node.children(&mut def_node.walk()).find_map(|child| {
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
        _ => def_node
            .child_by_field_name("name")
            .map(|name_node| name_node.range()),
    };
    if let Some(inner_name_range) = inner_name_range {
        name_range = inner_name_range;
    }

    let target_preprocessing_results = sema.preprocess_file(file_id);
    let target_offsets = target_preprocessing_results.offsets();
    let navs = vec![NavigationTarget {
        file_id,
        full_range: s_range_to_u_range(target_offsets, ts_range_to_lsp_range(&def_node.range())),
        focus_range: s_range_to_u_range(target_offsets, ts_range_to_lsp_range(&name_range)).into(),
    }];

    RangeInfo::new(u_range, navs).into()
}

/// Convert a position seen by the user to a position seen by the server (preprocessed).
fn u_pos_to_s_pos(
    offsets: &FxHashMap<u32, Vec<Offset>>,
    u_pos: lsp_types::Position,
) -> lsp_types::Position {
    if let Some(diff) = offsets.get(&u_pos.line).map(|offsets| {
        offsets
            .iter()
            .filter(|offset| offset.range.end.character <= u_pos.character)
            .map(|offset| offset.diff)
            .sum::<i32>()
    }) {
        lsp_types::Position {
            line: u_pos.line,
            character: u_pos.character.saturating_add_signed(diff),
        }
    } else {
        u_pos
    }
}

/// Convert a range seen by the server to a range seen by the user.
fn s_range_to_u_range(
    offsets: &FxHashMap<u32, Vec<Offset>>,
    mut s_range: lsp_types::Range,
) -> lsp_types::Range {
    if let Some(diff) = offsets.get(&s_range.start.line).map(|offsets| {
        offsets
            .iter()
            .filter(|offset| offset.range.start.character <= s_range.start.character)
            .map(|offset| offset.diff)
            .sum::<i32>()
    }) {
        s_range.end.character = s_range.end.character.saturating_add_signed(-diff);
        s_range.start.character = s_range.start.character.saturating_add_signed(-diff);
    }

    s_range
}
