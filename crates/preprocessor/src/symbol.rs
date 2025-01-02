use std::hash::Hash;

use deepsize::DeepSizeOf;
use smol_str::SmolStr;
use sourcepawn_lexer::{Delta, Symbol, TextRange, TokenKind};

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

#[allow(clippy::from_over_into)]
impl Into<Symbol> for &RangeLessSymbol {
    fn into(self) -> Symbol {
        Symbol::new(
            self.token_kind,
            Some(&self.text),
            TextRange::default(),
            self.delta,
        )
    }
}
