use smol_str::SmolStr;
use sourcepawn_lexer::Symbol;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    symbol: Symbol,
    original_range: Option<lsp_types::Range>,
}

impl From<Symbol> for Token {
    fn from(symbol: Symbol) -> Self {
        Self {
            symbol,
            original_range: None,
        }
    }
}

impl Token {
    pub fn inline_text(&self) -> SmolStr {
        self.symbol.inline_text()
    }

    pub fn text(&self) -> SmolStr {
        self.symbol.text()
    }

    pub fn range(&self) -> &lsp_types::Range {
        &self.symbol.range
    }

    pub fn delta(&self) -> &sourcepawn_lexer::Delta {
        &self.symbol.delta
    }

    pub fn set_delta(&mut self, delta: sourcepawn_lexer::Delta) {
        self.symbol.delta = delta;
    }

    pub fn original_range(&self) -> Option<lsp_types::Range> {
        self.original_range
    }

    pub fn set_original_range(&mut self, range: lsp_types::Range) {
        self.original_range = Some(range);
    }

    pub fn symbol(&self) -> &Symbol {
        &self.symbol
    }

    pub fn token_kind(&self) -> sourcepawn_lexer::TokenKind {
        self.symbol.token_kind
    }
}
