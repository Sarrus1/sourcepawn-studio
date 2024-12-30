//! This module handles the "translation" between a "user seen" position to a
//! "server seen" position.
//!
//! * "User seen" is the representation of the source file in the editor, i.e,
//!   what the user sees when they open the file in the editor. In the code, we
//!   mark "user seen" positions and ranges with the prefix `u_`.
//! * "Server seen" is the representation of the source file once it has been
//!   preprocessed by the server. In the code, we mark "server seen" positions
//!   and ranges with the prefix `s_`.
//!
//! We explain the reason for this distinction with the following example:
//! ```cpp
//! // What the user sees
//! #define FOOOOO foo
//! int bar = 1;
//! int FOOOOO = bar;
//!
//! // What the server sees
//! #define FOOOOO foo
//! int bar = 1;
//! int foo = bar;
//! ```
//!
//! Since `FOOOOO` has 6 characters but gets replaced with `foo` which only has
//! 3, this introduces an offset between the 2 representations. Therefore, when
//! the client requests a GoToDefinition on `bar`, the server sees that the
//! client requested a GoToDefinition on some whitespace.
//!
//! This modules introduces several data structures, most notably [SourceMap]
//! which allows to go back and forth between a user range and a server range.
//!

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

    name_len: TextSize,

    /// The index of the macro that was expanded.
    idx: u32,

    /// The [`file_id`](FileId) of the file containing the macro that was expanded.
    file_id: FileId,
}

impl ExpandedSymbolOffset {
    pub fn range(&self) -> &TextRange {
        &self.range
    }

    pub fn name_range(&self) -> TextRange {
        TextRange::at(self.range.start(), self.name_len)
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
            name_len: TextSize::new(macro_.name_len as u32),
            file_id: macro_.file_id,
        });
        // self.push_new_range(
        //     range,
        //     TextRange::new(start_offset.into(), end_offset.into()),
        // );
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
        let expanded_symbol = self.expanded_symbols[idx].clone();
        if expanded_symbol.name_len + expanded_symbol.range.start() < u_pos {
            return None;
        }
        Some(expanded_symbol)
    }

    pub fn expanded_symbol_from_s_pos(&self, s_pos: TextSize) -> Option<ExpandedSymbolOffset> {
        let idx = self
            .expanded_symbols
            .binary_search_by(|symbol| {
                if symbol.expanded_range.start() > s_pos {
                    std::cmp::Ordering::Greater
                } else if symbol.expanded_range.end() < s_pos {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .ok()?;
        Some(self.expanded_symbols[idx].clone())
    }

    pub fn closest_s_position(&self, u_pos: TextSize) -> TextSize {
        if let Some(symbol) = self.expanded_symbol_from_u_pos(u_pos) {
            return symbol.expanded_range.start(); // FIXME: ?
        }
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

    pub fn closest_u_position(&self, s_pos: TextSize, end: bool) -> TextSize {
        let idx = self
            .s_range_to_u_range
            .binary_search_by(|&(s_range_idx, _)| {
                let s_range = self.arena[s_range_idx];
                if end {
                    s_range.end().cmp(&s_pos)
                } else {
                    s_range.start().cmp(&s_pos)
                }
            })
            .unwrap_or_default();
        let (s_range_idx, u_range_idx) = self.s_range_to_u_range[idx];
        if !self.arena[s_range_idx].contains_inclusive(s_pos) {
            if let Some(expanded_symbol) = self.expanded_symbol_from_s_pos(s_pos) {
                return if end {
                    expanded_symbol.name_range().end()
                } else {
                    expanded_symbol.name_range().start()
                };
            }
        }
        if end {
            let delta = s_pos
                .checked_sub(self.arena[s_range_idx].end())
                .unwrap_or_default();
            self.arena[u_range_idx]
                .end()
                .checked_add(delta)
                .unwrap_or_default()
        } else {
            let delta = s_pos
                .checked_sub(self.arena[s_range_idx].start())
                .unwrap_or_default();
            self.arena[u_range_idx]
                .start()
                .checked_add(delta)
                .unwrap_or_default()
        }
    }

    pub fn closest_u_range(&self, s_range: TextRange) -> TextRange {
        let start = self.closest_u_position(s_range.start(), false);
        let end = self.closest_u_position(s_range.end(), true);
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

    pub fn print_u_range_to_s_range(&self) {
        for (i, j) in self.u_range_to_s_range.iter().cloned() {
            let u = self.arena[i];
            let s = self.arena[j];
            let u_start: u32 = u.start().into();
            let u_end: u32 = u.end().into();
            let s_start: u32 = s.start().into();
            let s_end: u32 = s.end().into();
            eprintln!("({}, {}) - ({}, {})", u_start, u_end, s_start, s_end);
        }
    }

    pub fn print_s_range_to_u_range(&self) {
        for (i, j) in self.s_range_to_u_range.iter().cloned() {
            let s = self.arena[i];
            let u = self.arena[j];
            let u_start: u32 = u.start().into();
            let u_end: u32 = u.end().into();
            let s_start: u32 = s.start().into();
            let s_end: u32 = s.end().into();
            eprintln!("({}, {}) - ({}, {})", s_start, s_end, u_start, u_end);
        }
    }
}
