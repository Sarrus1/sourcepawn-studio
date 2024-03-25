use vfs::FileId;

/// Offset induced by a macro expansion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offset {
    /// The range of the symbol that was expanded.
    pub range: lsp_types::Range,

    /// The difference in characters between the original symbol and the expanded symbol.
    pub diff: i32,

    /// The difference introduced by the macro arguments.
    ///
    /// # Example
    /// ```c
    /// #define FOO(x) x
    /// int foo = FOO(1);
    /// ```
    ///
    /// In this example, the `args_diff` would be `3` because the `(1)` has a length of `3`.
    pub args_diff: u32,

    /// The index of the macro that was expanded.
    pub idx: u32,

    /// The [`file_id`](FileId) of the file containing the macro that was expanded.
    pub file_id: FileId,
}

impl Offset {
    pub fn contains(&self, pos: lsp_types::Position) -> bool {
        if self.range.start.line > pos.line || self.range.end.line < pos.line {
            return false;
        }

        if self.range.start.line == pos.line && self.range.start.character > pos.character
            || self.range.end.line == pos.line && self.range.end.character < pos.character
        {
            return false;
        }

        true
    }
}
