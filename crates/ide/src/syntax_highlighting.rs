use std::fmt::{self, Debug, Write};

use hir::Semantics;
use ide_db::{RootDatabase, SymbolKind};
use itertools::Itertools;
use syntax::utils::{intersect, ts_range_to_lsp_range};
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
    // Determine the root based on the given range.
    let tree = sema.parse(file_id);
    let (_root, range_to_highlight) = {
        let source_file = tree.root_node();
        match range_to_highlight {
            Some(range) => {
                let node = match tree.covering_element(range) {
                    Some(it) => it,
                    None => source_file,
                };
                (node, range)
            }
            None => (source_file, ts_range_to_lsp_range(&source_file.range())),
        }
    };
    let mut res = Vec::new();
    let preprocessing_res = sema.preprocess_file(file_id);
    for (_, offsets) in preprocessing_res
        .offsets()
        .iter()
        .sorted_by_key(|(k, _)| *k)
    {
        offsets
            .iter()
            .filter(|offset| intersect(offset.range, range_to_highlight).is_some())
            .for_each(|offset| {
                res.push(HlRange {
                    range: offset.range,
                    highlight: Highlight {
                        tag: HlTag::Symbol(SymbolKind::Macro),
                        mods: HlMods::default(),
                    },
                });
            });
    }

    res
}
