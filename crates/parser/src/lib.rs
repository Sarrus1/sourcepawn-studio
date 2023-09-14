use fxhash::FxHashMap;
use lazy_static::lazy_static;
use lsp_types::{Range, Url};
use parking_lot::RwLock;
use preprocessor::Offset;
use std::sync::Arc;
use syntax::{comment::Comment, deprecated::Deprecated, FileId, SPItem};
use tree_sitter::Query;

pub mod comment_parser;
pub mod define_parser;
pub mod enum_parser;
pub mod enum_struct_parser;
pub mod function_parser;
pub mod include_parser;
pub mod macro_parser;
pub mod methodmap_parser;
pub mod property_parser;
pub mod typedef_parser;
pub mod typeset_parser;
pub mod variable_parser;

lazy_static! {
    static ref VARIABLE_QUERY: Query = {
        Query::new(tree_sitter_sourcepawn::language(), "[(variable_declaration_statement) @declaration.variable (old_variable_declaration_statement)  @declaration.variable]").expect("Could not build variable query.")
    };
    pub(crate) static ref ERROR_QUERY: Query =
        Query::new(tree_sitter_sourcepawn::language(), "(ERROR) @error")
            .expect("Could not build error query.");
}

pub struct Parser<'a> {
    pub comments: Vec<Comment>,
    pub deprecated: Vec<Deprecated>,
    pub anon_enum_counter: u32,
    pub sp_items: &'a mut Vec<Arc<RwLock<SPItem>>>,
    pub declarations: &'a mut FxHashMap<String, Arc<RwLock<SPItem>>>,
    pub offsets: &'a FxHashMap<u32, Vec<Offset>>,
    pub source: &'a String,
    pub uri: Arc<Url>,
    pub file_id: FileId,
}

pub fn build_v_range(offsets: &FxHashMap<u32, Vec<Offset>>, range: &Range) -> Range {
    let mut start = range.start;
    let mut end = range.end;

    if let Some(start_offsets) = offsets.get(&start.line) {
        for offset in start_offsets.iter() {
            if offset.col < start.character {
                start.character = start
                    .character
                    .checked_add_signed(-offset.diff)
                    .unwrap_or(0);
            }
        }
    }

    if let Some(end_offsets) = offsets.get(&end.line) {
        for offset in end_offsets.iter() {
            if offset.col < end.character {
                end.character = end.character.checked_add_signed(-offset.diff).unwrap_or(0);
            }
        }
    }

    Range { start, end }
}

impl<'a> Parser<'a> {
    pub fn build_v_range(&self, range: &Range) -> Range {
        build_v_range(self.offsets, range)
    }
}
