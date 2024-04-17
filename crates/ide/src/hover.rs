mod render;

use hir::{HasSource, Semantics};
use ide_db::{Documentation, RootDatabase};
use preprocessor::{db::PreprocDatabase, PreprocessingResult};
use syntax::utils::{lsp_position_to_ts_point, ts_range_to_lsp_range};

use crate::{
    goto_definition::find_macro_def, markup::Markup, s_range_to_u_range, u_pos_to_s_pos,
    FilePosition, RangeInfo,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HoverConfig {
    pub links_in_hover: bool,
    pub documentation: bool,
    pub keywords: bool,
    pub format: HoverDocFormat,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HoverDocFormat {
    Markdown,
    PlainText,
}

#[derive(Debug, Clone)]
pub enum HoverAction {
    // Runnable(Runnable),
    Implementation(FilePosition),
    Reference(FilePosition),
}

// impl HoverAction {
//     fn goto_type_from_targets(db: &RootDatabase, targets: Vec<hir::ModuleDef>) -> Self {
//         let targets = targets
//             .into_iter()
//             .filter_map(|it| {
//                 Some(HoverGotoTypeData {
//                     mod_path: render::path(
//                         db,
//                         it.module(db)?,
//                         it.name(db).map(|name| name.display(db).to_string()),
//                     ),
//                     nav: it.try_to_nav(db)?,
//                 })
//             })
//             .collect();
//         HoverAction::GoToType(targets)
//     }
// }

/// Contains the results when hovering over an item
#[derive(Debug, Default)]
pub struct HoverResult {
    pub markup: Markup,
    pub actions: Vec<HoverAction>,
}

pub(crate) fn hover(
    db: &RootDatabase,
    mut fpos: FilePosition,
    config: &HoverConfig,
) -> Option<RangeInfo<HoverResult>> {
    let sema = &Semantics::new(db);
    let preprocessing_results = sema.preprocess_file(fpos.file_id);
    let offsets = preprocessing_results.offsets();
    let tree = sema.parse(fpos.file_id);
    let root_node = tree.root_node();
    if let Some(hover) = find_macro_hover(&preprocessing_results, sema, &fpos.position) {
        return Some(hover);
    }

    let source_u_range = u_pos_to_s_pos(
        preprocessing_results.args_map(),
        offsets,
        &mut fpos.position,
    );

    let node = root_node.descendant_for_point_range(
        lsp_position_to_ts_point(&fpos.position),
        lsp_position_to_ts_point(&fpos.position),
    )?;
    let def = sema.find_def(fpos.file_id, &node)?;
    let u_range = match source_u_range {
        Some(u_range) => u_range,
        None => s_range_to_u_range(offsets, ts_range_to_lsp_range(&node.range())),
    };

    let file_id = def.file_id(db);
    let source_tree = sema.parse(file_id);
    let text = db.preprocessed_text(file_id);
    let render = render::render_def(db, def.clone())?;
    let def_node = def.source(db, &source_tree)?.value;

    if !config.documentation {
        let res = HoverResult {
            markup: Markup::fenced_block(render),
            actions: vec![],
        };
        return Some(RangeInfo::new(u_range, res));
    }
    if let Some(docs) = Documentation::from_node(def_node, text.as_bytes()) {
        let res = HoverResult {
            markup: Markup::from(format!(
                "{}\n\n---\n\n{}",
                Markup::fenced_block(render),
                Markup::from(docs.to_markdown()),
            )),
            actions: vec![],
        };
        return Some(RangeInfo::new(u_range, res));
    }
    let res = HoverResult {
        markup: Markup::fenced_block(render),
        actions: vec![],
    };
    Some(RangeInfo::new(u_range, res))
}

fn find_macro_hover(
    preprocessing_results: &PreprocessingResult,
    sema: &Semantics<RootDatabase>,
    pos: &lsp_types::Position,
) -> Option<RangeInfo<HoverResult>> {
    let (offset, def) = find_macro_def(preprocessing_results.offsets(), pos, sema)?;
    let offsets = preprocessing_results.offsets();
    let preprocessed_text = preprocessing_results.preprocessed_text();
    let file_id = def.file_id(sema.db);
    let source_tree = sema.parse(file_id);
    let def_node = def.source(sema.db, &source_tree)?.value;
    let source_text = sema.db.preprocessed_text(file_id);
    let source_text = def_node.utf8_text(source_text.as_bytes()).ok()?;

    let mut start = offset.range.start.character;
    let mut end = offset.range.end.character;
    offsets[&pos.line]
        .iter()
        .filter(|prev_offset| prev_offset.range.start.character < offset.range.start.character)
        .for_each(|prev_offset| {
            start = start.saturating_add_signed(
                prev_offset
                    .diff
                    .saturating_sub_unsigned(prev_offset.args_diff),
            );
            end = end.saturating_add_signed(
                prev_offset
                    .diff
                    .saturating_sub_unsigned(prev_offset.args_diff),
            );
        });
    end = end.saturating_add_signed(offset.diff);
    let start = start as usize;
    let end = end as usize;
    let slc = start..end;
    // The preprocessed file might be shorter than the original file
    let hover_text = preprocessed_text
        .lines()
        .nth(pos.line as usize)
        .and_then(|it| it.get(slc))
        .map(|it| it.to_string());
    match hover_text {
        Some(hover_text) => {
            let res = HoverResult {
                markup: Markup::from(format!(
                    "{}\nExpands to:\n{}",
                    Markup::fenced_block(source_text),
                    Markup::fenced_block(hover_text.trim())
                )),
                actions: vec![],
            };
            Some(RangeInfo::new(offset.range, res))
        }
        None => {
            let res = HoverResult {
                markup: Markup::fenced_block(source_text),
                actions: vec![],
            };
            Some(RangeInfo::new(offset.range, res))
        }
    }
}
