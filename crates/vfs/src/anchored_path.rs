use crate::FileId;

/// Url relative to a file.
///
/// Owned version of [`AnchoredUrl`].
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AnchoredUrlBuf {
    /// File that this path is relative to.
    pub anchor: FileId,
    /// Uri relative to `anchor`'s containing directory.
    pub uri: String,
}

/// Url relative to a file.
///
/// Borrowed version of [`AnchoredUrlBuf`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AnchoredUrl<'a> {
    /// File that this path is relative to.
    pub anchor: FileId,
    /// Uri relative to `anchor`'s containing directory.
    pub uri: &'a str,
}

impl<'a> AnchoredUrl<'a> {
    /// Create a new [`AnchoredUrl`] from `anchor` and `uri`.
    pub fn new(anchor: FileId, uri: &str) -> AnchoredUrl<'_> {
        AnchoredUrl { anchor, uri }
    }
}
