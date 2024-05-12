use hir::DefResolution;
use ide_db::{Documentation, SymbolKind};
use smol_str::SmolStr;

/// `CompletionItem` describes a single completion entity which expands to 1 or more entries in the
/// editor pop-up. It is basically a POD with various properties.
#[derive(Clone, Default, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct CompletionItem {
    /// Label in the completion pop up which identifies completion.
    pub label: SmolStr,

    /// What item (struct, function, etc) are we completing.
    pub kind: CompletionKind,

    /// What to insert if completion is accepted.
    pub insert_text: Option<String>,

    /// Additional info to show in the UI pop up.
    pub detail: Option<String>,

    /// Documentation to show in the UI pop up.
    pub documentation: Option<Documentation>,

    /// Whether this item is marked as deprecated
    pub deprecated: bool,

    /// If completing a function call, ask the editor to show parameter popup
    /// after completion.
    pub trigger_call_info: bool,

    pub data: Option<DefResolution>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum CompletionKind {
    SymbolKind(SymbolKind),
    Snippet,
    #[default]
    Keyword,
    Literal,
    Directory,
    File,
}

impl From<SymbolKind> for CompletionKind {
    fn from(kind: SymbolKind) -> Self {
        Self::SymbolKind(kind)
    }
}
