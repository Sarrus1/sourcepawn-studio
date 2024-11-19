use std::hash::Hash;

use deepsize::DeepSizeOf;
use smol_str::SmolStr;
use sourcepawn_lexer::{Delta, Symbol, TextRange, TextSize, TokenKind};

/// Wrapper around `Symbol` that does not contain range information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeLessSymbol {
    pub(crate) token_kind: TokenKind,
    text: SmolStr,
    pub(crate) delta: Delta,
}

impl DeepSizeOf for RangeLessSymbol {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        self.text.deep_size_of_children(context)
    }
}

impl Hash for RangeLessSymbol {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.token_kind.hash(state);
        self.text.hash(state);
        self.delta.hash(state);
    }
}

impl From<Symbol> for RangeLessSymbol {
    fn from(symbol: Symbol) -> Self {
        Self {
            token_kind: symbol.token_kind,
            text: symbol.inline_text(), // TODO: Maybe use an option here?
            delta: symbol.delta,
        }
    }
}

impl RangeLessSymbol {
    pub fn to_symbol(&self, prev_range: TextRange) -> Symbol {
        let prev_end: u32 = prev_range.end().into();
        let range = TextRange::at(
            TextSize::new(prev_end.saturating_add_signed(self.delta)),
            TextSize::new(self.text.len() as u32), // FIXME: Is this wrong?
        );
        Symbol::new(self.token_kind, Some(&self.text), range, self.delta)
    }
}
