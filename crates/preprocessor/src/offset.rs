use std::cmp::Ordering;

use sourcepawn_lexer::{TextRange, TextSize};
use vfs::FileId;

use crate::macros::Macro;

/// Offset induced by a macro expansion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpandedSymbolOffset {
    /// The range of the symbol that was expanded.
    pub range: TextRange,

    /// The range of the expanded text.
    pub expanded_range: TextRange,

    /// The index of the macro that was expanded.
    pub idx: u32,

    /// The [`file_id`](FileId) of the file containing the macro that was expanded.
    pub file_id: FileId,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceMap {
    vec: Vec<(TextRange, TextRange)>,
    expanded_symbols: Vec<ExpandedSymbolOffset>,
}

impl SourceMap {
    pub fn push_new_range(&mut self, u_range: TextRange, s_range: TextRange) {
        self.vec.push((u_range, s_range));
    }

    pub fn push_expanded_symbol(
        &mut self,
        range: TextRange,
        expanded_range: TextRange,
        macro_: &Macro,
    ) {
        self.expanded_symbols.push(ExpandedSymbolOffset {
            range,
            expanded_range,
            idx: macro_.idx,
            file_id: macro_.file_id,
        });
    }

    pub fn expanded_symbol_from_u_pos(&self, u_pos: TextSize) -> Option<ExpandedSymbolOffset> {
        let idx = self
            .expanded_symbols
            .binary_search_by(|prob| match prob.range.start().cmp(&u_pos) {
                Ordering::Greater => Ordering::Greater,
                Ordering::Equal => Ordering::Equal,
                Ordering::Less if prob.range.end().cmp(&u_pos) != Ordering::Less => Ordering::Equal,
                Ordering::Less => Ordering::Less,
            })
            .ok()?;
        Some(self.expanded_symbols[idx].clone())
    }

    pub fn closest_s_position(&self, u_pos: TextSize) -> TextSize {
        let idx = self.vec.partition_point(|&ranges| ranges.0.contains(u_pos));
        let delta = u_pos
            .checked_sub(self.vec[idx].0.start())
            .unwrap_or_default();
        self.vec[idx]
            .1
            .start()
            .checked_add(delta)
            .unwrap_or_default()
    }

    pub fn closest_u_position(&self, s_pos: TextSize) -> TextSize {
        let idx = self.vec.partition_point(|&ranges| ranges.1.contains(s_pos));
        let delta = s_pos
            .checked_sub(self.vec[idx].1.start())
            .unwrap_or_default();
        self.vec[idx]
            .0
            .start()
            .checked_add(delta)
            .unwrap_or_default()
    }

    pub fn shrink_to_fit(&mut self) {
        self.vec.shrink_to_fit();
        self.expanded_symbols.shrink_to_fit();
    }

    pub fn sort(&mut self) {
        self.vec.sort_by(|a, b| a.0.ordering(b.0)); // FIXME: This might be wrong
        self.expanded_symbols
            .sort_by(|a, b| a.range.ordering(b.range));
    }
}
