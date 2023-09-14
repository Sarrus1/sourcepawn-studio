//! Maps [uris](Url) to compact integer ids. We don't care about clearings uris which
//! no longer exist -- the assumption is total size of uris we ever look at is
//! not too big.

use fxhash::FxHasher;
use lsp_types::Url;
use syntax::FileId;

use std::hash::BuildHasherDefault;

use indexmap::IndexSet;

/// Structure to map between [`VfsPath`] and [`FileId`].
#[derive(Default, Debug, Clone)]
pub struct PathInterner {
    map: IndexSet<Url, BuildHasherDefault<FxHasher>>,
}

impl PathInterner {
    /// Get the id corresponding to `path`.
    ///
    /// If `path` does not exists in `self`, returns [`None`].
    pub fn get(&self, uri: &Url) -> Option<FileId> {
        self.map.get_index_of(uri).map(|i| FileId(i as u32))
    }

    /// Insert `path` in `self`.
    ///
    /// - If `path` already exists in `self`, returns its associated id;
    /// - Else, returns a newly allocated id.
    pub fn intern(&mut self, uri: Url) -> FileId {
        let (id, _added) = self.map.insert_full(uri);
        assert!(id < u32::MAX as usize);
        FileId(id as u32)
    }

    /// Returns the path corresponding to `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` does not exists in `self`.
    pub fn lookup(&self, id: FileId) -> &Url {
        self.map.get_index(id.0 as usize).unwrap()
    }
}
