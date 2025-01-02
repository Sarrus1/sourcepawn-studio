use hir::Function;
use line_index::TextRange;
use smol_str::SmolStr;
use vfs::FileId;

use crate::SymbolKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallItem {
    pub name: SmolStr,
    pub kind: SymbolKind,
    pub deprecated: bool,
    pub details: Option<String>,
    pub file_id: FileId,
    pub full_range: TextRange,
    pub focus_range: Option<TextRange>,
    pub data: Option<Function>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncomingCallItem {
    pub call_item: CallItem,
    pub ranges: Vec<TextRange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutgoingCallItem {
    pub call_item: CallItem,
    pub ranges: Vec<TextRange>,
}
