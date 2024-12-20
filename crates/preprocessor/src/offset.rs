use itertools::Itertools;
use la_arena::{Arena, Idx};
use sourcepawn_lexer::{TextRange, TextSize};
use vfs::FileId;

use crate::macros::Macro;

/// Offset induced by a macro expansion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpandedSymbolOffset {
    /// The range of the symbol that was expanded.
    range: TextRange,

    /// The range of the expanded text.
    expanded_range: TextRange,

    /// The index of the macro that was expanded.
    idx: u32,

    /// The [`file_id`](FileId) of the file containing the macro that was expanded.
    file_id: FileId,
}

impl ExpandedSymbolOffset {
    pub fn range(&self) -> &TextRange {
        &self.range
    }

    pub fn expanded_range(&self) -> &TextRange {
        &self.expanded_range
    }

    pub fn idx(&self) -> u32 {
        self.idx
    }

    pub fn file_id(&self) -> FileId {
        self.file_id
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceMap {
    arena: Arena<TextRange>,
    u_range_to_s_range: Vec<(Idx<TextRange>, Idx<TextRange>)>,
    s_range_to_u_range: Vec<(Idx<TextRange>, Idx<TextRange>)>,
    expanded_symbols: Vec<ExpandedSymbolOffset>,
}

impl SourceMap {
    pub fn push_new_range(&mut self, u_range: TextRange, s_range: TextRange) {
        let u_range_idx = self.arena.alloc(u_range);
        let s_range_idx = self.arena.alloc(s_range);
        self.u_range_to_s_range.push((u_range_idx, s_range_idx));
        self.s_range_to_u_range.push((s_range_idx, u_range_idx));
    }

    pub fn push_expanded_symbol(
        &mut self,
        range: TextRange,
        start_offset: u32,
        end_offset: u32,
        macro_: &Macro,
    ) {
        self.expanded_symbols.push(ExpandedSymbolOffset {
            range,
            expanded_range: TextRange::new(start_offset.into(), end_offset.into()),
            idx: macro_.idx,
            file_id: macro_.file_id,
        });
    }

    pub fn expanded_symbol_from_u_pos(&self, u_pos: TextSize) -> Option<ExpandedSymbolOffset> {
        let idx = self
            .expanded_symbols
            .binary_search_by(|symbol| {
                if symbol.range.start() > u_pos {
                    std::cmp::Ordering::Greater
                } else if symbol.range.end() < u_pos {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .ok()?;
        Some(self.expanded_symbols[idx].clone())
    }

    pub fn closest_s_position(&self, u_pos: TextSize) -> TextSize {
        let idx = self
            .u_range_to_s_range
            .binary_search_by(|&(u_range_idx, _)| {
                let u_range = self.arena[u_range_idx];
                if u_range.start() > u_pos {
                    std::cmp::Ordering::Greater
                } else if u_range.end() < u_pos {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .unwrap_or_default();
        let (u_range_idx, s_range_idx) = self.u_range_to_s_range[idx];
        let delta = u_pos
            .checked_sub(self.arena[u_range_idx].start())
            .unwrap_or_default();
        self.arena[s_range_idx]
            .start()
            .checked_add(delta)
            .unwrap_or_default()
    }

    pub fn closest_u_position(&self, s_pos: TextSize) -> TextSize {
        let idx = self
            .s_range_to_u_range
            .binary_search_by(|&(s_range_idx, _)| {
                let s_range = self.arena[s_range_idx];
                if s_range.start() > s_pos {
                    std::cmp::Ordering::Greater
                } else if s_range.end() < s_pos {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .unwrap_or_default();
        let (s_range_idx, u_range_idx) = self.s_range_to_u_range[idx];
        let delta = s_pos
            .checked_sub(self.arena[s_range_idx].start())
            .unwrap_or_default();
        self.arena[u_range_idx]
            .start()
            .checked_add(delta)
            .unwrap_or_default()
    }

    pub fn closest_u_range(&self, s_range: TextRange) -> TextRange {
        let start = self.closest_u_position(s_range.start());
        let end = self.closest_u_position(s_range.end());
        TextRange::new(start, end)
    }

    pub fn shrink_to_fit(&mut self) {
        self.arena.shrink_to_fit();
        self.u_range_to_s_range.shrink_to_fit();
        self.s_range_to_u_range.shrink_to_fit();
        self.expanded_symbols.shrink_to_fit();
    }

    pub fn sort(&mut self) {
        self.u_range_to_s_range
            .sort_by(|a, b| self.arena[a.0].ordering(self.arena[b.0]));
        self.s_range_to_u_range
            .sort_by(|a, b| self.arena[a.0].ordering(self.arena[b.0]));
        self.expanded_symbols
            .sort_by(|a, b| a.range.ordering(b.range));
    }

    pub fn u_range_to_s_range_vec(&self) -> Vec<(TextRange, TextRange)> {
        self.u_range_to_s_range
            .iter()
            .map(|(u_range_idx, s_range_idx)| (self.arena[*u_range_idx], self.arena[*s_range_idx]))
            .collect_vec()
    }

    pub fn expanded_symbols(&self) -> &[ExpandedSymbolOffset] {
        &self.expanded_symbols
    }

    pub fn arena_len(&self) -> usize {
        self.arena.len()
    }
}
