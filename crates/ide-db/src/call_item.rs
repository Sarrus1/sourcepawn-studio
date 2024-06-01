use hir::Function;
use lsp_types::Range;
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
    pub full_range: Range,
    pub focus_range: Option<Range>,
    pub data: Option<Function>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncomingCallItem {
    pub call_item: CallItem,
    pub ranges: Vec<Range>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutgoingCallItem {
    pub call_item: CallItem,
    pub ranges: Vec<Range>,
}
