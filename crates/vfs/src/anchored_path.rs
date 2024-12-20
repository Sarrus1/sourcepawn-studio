use crate::FileId;

/// Path relative to a file.
///
/// Owned version of [`AnchoredUrl`].
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AnchoredPathBuf {
    /// File that this path is relative to.
    pub anchor: FileId,
    /// Path relative to `anchor`'s containing directory.
    pub path: String,
}

/// Path relative to a file.
///
/// Borrowed version of [`AnchoredUrlBuf`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AnchoredPath<'a> {
    /// File that this path is relative to.
    pub anchor: FileId,
    /// Path relative to `anchor`'s containing directory.
    pub path: &'a str,
}

impl AnchoredPath<'_> {
    /// Create a new [`AnchoredUrl`] from `anchor` and `path`.
    pub fn new(anchor: FileId, path: &str) -> AnchoredPath<'_> {
        AnchoredPath { anchor, path }
    }
}
