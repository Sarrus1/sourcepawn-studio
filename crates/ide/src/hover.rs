use hir::{DefResolution, HasSource, Semantics};
use ide_db::RootDatabase;
use preprocessor::db::PreprocDatabase;

use crate::{markup::Markup, FilePosition, RangeInfo};

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
    fpos: FilePosition,
    _config: &HoverConfig,
) -> Option<RangeInfo<HoverResult>> {
    let sema = &Semantics::new(db);
    let preprocessing_results = sema.preprocess_file(fpos.file_id);
    let offsets = preprocessing_results.offsets();
    if let Some(offset) = offsets
        .get(&fpos.position.line)
        .and_then(|offsets| offsets.iter().find(|offset| offset.contains(fpos.position)))
    {
        let preprocessed_text = preprocessing_results.preprocessed_text();
        let def = sema
            .find_macro_def(offset.file_id, offset.idx)
            .map(DefResolution::from)?;
        let file_id = def.file_id(db);
        let source_tree = sema.parse(file_id);
        let def_node = def.source(db, &source_tree)?.value;
        let source_text = db.preprocessed_text(file_id);
        let source_text = def_node.utf8_text(source_text.as_bytes()).ok()?;

        let mut start = offset.range.start.character;
        let mut end = offset.range.end.character;
        offsets[&fpos.position.line]
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
            .nth(fpos.position.line as usize)
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
                return Some(RangeInfo::new(offset.range, res));
            }
            None => {
                let res = HoverResult {
                    markup: Markup::fenced_block(source_text),
                    actions: vec![],
                };
                return Some(RangeInfo::new(offset.range, res));
            }
        }
    }

    None
}
