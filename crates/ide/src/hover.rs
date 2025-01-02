mod actions;
mod render;

use std::panic::AssertUnwindSafe;

use hir::{DefResolution, HasSource, Semantics};
use ide_db::{Documentation, RootDatabase};
use itertools::Itertools;
use preprocessor::{db::PreprocDatabase, PreprocessingResult};
use smol_str::ToSmolStr;
use syntax::utils::ts_range_to_text_range;
use vfs::FileId;

use crate::{
    events::{event_hover, event_name},
    goto_definition::find_inner_name_range,
    markup::Markup,
    FilePosition, NavigationTarget, RangeInfo,
};

use self::actions::goto_type_action_for_def;
pub(crate) use render::{render_def, Render};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HoverAction {
    // Runnable(Runnable),
    Implementation(FilePosition),
    Reference(FilePosition),
    GoToType(Vec<HoverGotoTypeData>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct HoverGotoTypeData {
    pub mod_path: String,
    pub nav: NavigationTarget,
}

impl HoverAction {
    fn goto_type_from_targets(db: &RootDatabase, targets: Vec<DefResolution>) -> Self {
        let mut targets = targets
            .into_iter()
            .filter_map(|def| {
                let sema = Semantics::new(db);
                let file_id = def.file_id(db);
                let source_tree = sema.parse(file_id);
                let name = def.name(db).map(|it| it.to_smolstr()).unwrap_or_default();
                let def_node = def.source(db, &source_tree)?.value;

                let name_range = find_inner_name_range(&def_node);

                let target_preprocessing_results = sema.preprocess_file(file_id);
                Some(HoverGotoTypeData {
                    mod_path: Default::default(),
                    nav: NavigationTarget {
                        name,
                        file_id,
                        full_range: target_preprocessing_results
                            .source_map()
                            .closest_u_range_always(ts_range_to_text_range(&def_node.range())),
                        focus_range: target_preprocessing_results
                            .source_map()
                            .closest_u_range_always(name_range)
                            .into(),
                    },
                })
            })
            .collect_vec();
        targets.dedup();
        HoverAction::GoToType(targets)
    }
}

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
    file_id_to_url: AssertUnwindSafe<&dyn Fn(FileId) -> Option<String>>,
    events_game_name: Option<&str>,
) -> Option<RangeInfo<HoverResult>> {
    let sema = &Semantics::new(db);
    let preprocessing_results = sema.preprocess_file(fpos.file_id);
    let tree = sema.parse(fpos.file_id);
    let root_node = tree.root_node();
    if let Some(hover) = find_macro_hover(&preprocessing_results, sema, &fpos) {
        return Some(hover);
    }
    fpos.offset = preprocessing_results
        .source_map()
        .closest_s_position_always(fpos.offset);

    let node =
        root_node.descendant_for_byte_range(fpos.raw_offset_usize(), fpos.raw_offset_usize())?;

    if let Some(name) = event_name(&node, &preprocessing_results.preprocessed_text()) {
        return event_hover(
            events_game_name,
            &name,
            &node,
            preprocessing_results.source_map(),
        );
    }

    let def = sema.find_def(fpos.file_id, &node)?;
    let u_range = preprocessing_results
        .source_map()
        .closest_u_range_always(ts_range_to_text_range(&node.range()));

    let file_id = def.file_id(db);
    let source_tree = sema.parse(file_id);
    let text = db.preprocessed_text(file_id);
    let render = render::render_def(db, def.clone())?;
    let mut actions = [goto_type_action_for_def(db, def.clone())]
        .into_iter()
        .flatten()
        .collect_vec();
    actions.dedup();
    let def_node = def.source(db, &source_tree)?.value;

    let markup = match render {
        Render::FileId(file_id) => Markup::from(file_id_to_url(file_id).unwrap_or_default()),
        Render::String(render) => Markup::fenced_block(render),
    };

    if !config.documentation {
        let res = HoverResult { markup, actions };
        return Some(RangeInfo::new(u_range, res));
    }
    if let Some(docs) = Documentation::from_node(def_node, text.as_bytes()) {
        let res = HoverResult {
            markup: Markup::from(format!(
                "{}\n\n---\n\n{}",
                markup,
                Markup::from(docs.to_markdown()),
            )),
            actions,
        };
        return Some(RangeInfo::new(u_range, res));
    }
    let res = HoverResult { markup, actions };
    Some(RangeInfo::new(u_range, res))
}

fn find_macro_hover(
    preprocessing_results: &PreprocessingResult,
    sema: &Semantics<RootDatabase>,
    fpos: &FilePosition,
) -> Option<RangeInfo<HoverResult>> {
    let (offset, def) = sema.find_macro_def(fpos)?;
    let preprocessed_text = preprocessing_results.preprocessed_text();
    let file_id = def.file_id(sema.db);
    let source_tree = sema.parse(file_id);
    let def_node = def.source(sema.db, &source_tree)?.value;
    let source = sema.db.preprocessed_text(file_id);
    let source_text = def_node.utf8_text(source.as_bytes()).ok()?;

    let start: u32 = offset.expanded_range().start().into();
    let end: u32 = offset.expanded_range().end().into();
    let slc = start as usize..end as usize;
    // The preprocessed file might be shorter than the original file
    let hover_text = preprocessed_text
        .get(slc)
        .map(String::from)
        .unwrap_or_default();

    let markup = Markup::from(format!(
        "{}\nExpands to:\n{}",
        Markup::fenced_block(source_text),
        Markup::fenced_block(hover_text.trim())
    ));

    let res = if let Some(docs) = Documentation::from_node(def_node, source.as_bytes()) {
        HoverResult {
            markup: Markup::from(format!(
                "{}\n\n---\n\n{}",
                markup,
                Markup::from(docs.to_markdown()),
            )),
            actions: vec![],
        }
    } else {
        HoverResult {
            markup,
            actions: vec![],
        }
    };

    Some(RangeInfo::new(offset.name_range(), res))
}
