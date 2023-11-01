//! Maps [`uris`](Url) to compact integer ids. We don't care about clearings uris which
//! no longer exist -- the assumption is total size of uris we ever look at is
//! not too big.

use fxhash::FxHasher;
use lsp_types::Url;

use std::{fmt::Display, hash::BuildHasherDefault};

use indexmap::IndexSet;

/// Handle to a file.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct FileId(pub u32);

/// safe because `FileId` is a newtype of `u32`
impl nohash_hasher::IsEnabled for FileId {}

impl From<u32> for FileId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl Display for FileId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
/// Structure to map between [`Url`] and [`FileId`].
#[derive(Default, Debug, Clone)]
pub struct PathInterner {
    map: IndexSet<Url, BuildHasherDefault<FxHasher>>,
}

impl PathInterner {
    /// Get the id corresponding to [`url`](Url).
    ///
    /// If [`url`](Url) does not exists in `self`, returns [`None`].
    pub fn get(&self, uri: &Url) -> Option<FileId> {
        self.map.get_index_of(uri).map(|i| FileId(i as u32))
    }

    /// Insert [`url`](Url) in `self`.
    ///
    /// - If [`url`](Url) already exists in `self`, returns its associated [`id`](FileId);
    /// - Else, returns a newly allocated [`id`](FileId).
    pub fn intern(&mut self, uri: Url) -> FileId {
        let (id, _added) = self.map.insert_full(uri);
        assert!(id < u32::MAX as usize);
        FileId(id as u32)
    }

    /// Returns the path corresponding to [`id`](FileId).
    ///
    /// # Panics
    ///
    /// Panics if [`id`](FileId) does not exists in `self`.
    pub fn lookup(&self, id: FileId) -> &Url {
        self.map.get_index(id.0 as usize).unwrap()
    }
}
