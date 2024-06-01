use std::fmt::{self, Debug, Write};

use hir::Semantics;
use hir_def::resolver::{HasResolver, ValueNs};
use ide_db::{RootDatabase, SymbolKind};
use itertools::Itertools;
use sourcepawn_lexer::{Literal, SourcepawnLexer, TokenKind};
use syntax::utils::intersect;
use vfs::FileId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Highlight {
    pub tag: HlTag,
    pub mods: HlMods,
}

impl fmt::Display for Highlight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.tag.fmt(f)?;
        for modifier in self.mods.iter() {
            f.write_char('.')?;
            modifier.fmt(f)?;
        }
        Ok(())
    }
}

impl From<HlTag> for Highlight {
    fn from(tag: HlTag) -> Highlight {
        Highlight::new(tag)
    }
}

impl From<SymbolKind> for Highlight {
    fn from(sym: SymbolKind) -> Highlight {
        Highlight::new(HlTag::Symbol(sym))
    }
}

impl Highlight {
    pub(crate) fn new(tag: HlTag) -> Highlight {
        Highlight {
            tag,
            mods: HlMods::default(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.tag == HlTag::None && self.mods.is_empty()
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HlMods(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum HlTag {
    Symbol(SymbolKind),

    BoolLiteral,
    StringLiteral,
    CharLiteral,
    FloatLiteral,
    IntLiteral,
    Comment,

    // For things which don't have a specific highlight.
    None,
}

// Don't forget to adjust the feature description in crates/ide/src/syntax_highlighting.rs.
// And make sure to use the lsp strings used when converting to the protocol in crates\rust-analyzer\src\semantic_tokens.rs, not the names of the variants here.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum HlMod {
    Macro,
}

impl HlMod {
    const ALL: &'static [HlMod; HlMod::Macro as usize + 1] = &[HlMod::Macro];

    #[allow(unused)]
    fn as_str(self) -> &'static str {
        match self {
            HlMod::Macro => "macro",
        }
    }

    fn mask(self) -> u32 {
        1 << (self as u32)
    }
}

impl HlMods {
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn contains(self, m: HlMod) -> bool {
        self.0 & m.mask() == m.mask()
    }

    pub fn iter(self) -> impl Iterator<Item = HlMod> {
        HlMod::ALL
            .iter()
            .copied()
            .filter(move |it| self.0 & it.mask() == it.mask())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct HlRange {
    pub range: lsp_types::Range,
    pub highlight: Highlight,
    // pub binding_hash: Option<u64>,
}

pub(crate) fn highlight(
    db: &RootDatabase,
    // config: HighlightConfig,
    file_id: FileId,
    range_to_highlight: Option<lsp_types::Range>,
) -> Vec<HlRange> {
    let sema = Semantics::new(db);
    let source = sema.file_text(file_id);
    let range_to_highlight = if let Some(range_to_highlight) = range_to_highlight {
        range_to_highlight
    } else {
        lsp_types::Range::new(
            lsp_types::Position {
                line: 0,
                character: 0,
            },
            lsp_types::Position {
                line: source.lines().count() as u32 + 1,
                character: 0,
            },
        )
    };
    let lexer = SourcepawnLexer::new(&source);
    let resolver = file_id.resolver(db);
    /* Ideally, we would want to use the AST here and do some range adjustments to get the user visible ranges.
    This would allow us to get proper modifiers for the symbols such as declarations or references.
    However, the current implementation is much simpler and should be good enough for now.
     */
    lexer
        .filter(|symbol| intersect(symbol.range, range_to_highlight).is_some())
        .flat_map(|symbol| match symbol.token_kind {
            TokenKind::Identifier => {
                let kind = match resolver.resolve_ident(&symbol.text())? {
                    ValueNs::MacroId(_) => SymbolKind::Macro,
                    ValueNs::LocalId(_) => SymbolKind::Local,
                    ValueNs::GlobalId(_) => SymbolKind::Global,
                    ValueNs::FunctionId(_) => SymbolKind::Function,
                    ValueNs::EnumStructId(_) => SymbolKind::EnumStruct,
                    ValueNs::MethodmapId(_) => SymbolKind::Methodmap,
                    ValueNs::EnumId(_) => SymbolKind::Enum,
                    ValueNs::VariantId(_) => SymbolKind::Variant,
                    ValueNs::TypedefId(_) => SymbolKind::Typedef,
                    ValueNs::TypesetId(_) => SymbolKind::Typeset,
                    ValueNs::FunctagId(_) => SymbolKind::Functag,
                    ValueNs::FuncenumId(_) => SymbolKind::Funcenum,
                    ValueNs::StructId(_) => SymbolKind::Struct,
                };
                Some(HlRange {
                    range: symbol.range,
                    highlight: Highlight {
                        tag: HlTag::Symbol(kind),
                        mods: HlMods::default(),
                    },
                })
            }
            TokenKind::True | TokenKind::False => Some(HlRange {
                range: symbol.range,
                highlight: Highlight::new(HlTag::BoolLiteral),
            }),
            TokenKind::Comment(_) => Some(HlRange {
                range: symbol.range,
                highlight: Highlight::new(HlTag::Comment),
            }),
            TokenKind::Literal(lit) => match lit {
                Literal::StringLiteral | Literal::CharLiteral => None, // FIXME: We can handle this but it overrides escaped characters.
                Literal::FloatLiteral => Some(HlRange {
                    range: symbol.range,
                    highlight: Highlight::new(HlTag::FloatLiteral),
                }),
                Literal::IntegerLiteral
                | Literal::BinaryLiteral
                | Literal::HexLiteral
                | Literal::OctodecimalLiteral => Some(HlRange {
                    range: symbol.range,
                    highlight: Highlight::new(HlTag::IntLiteral),
                }),
            },

            _ => None,
        })
        .collect_vec()
}
