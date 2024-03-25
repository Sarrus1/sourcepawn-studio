//! Maps [`uris`](Url) to compact integer ids. We don't care about clearings uris which
//! no longer exist -- the assumption is total size of uris we ever look at is
//! not too big.

use fxhash::FxHasher;

use std::hash::BuildHasherDefault;

use indexmap::IndexSet;

use crate::{vfs_path::VfsPath, FileId};

/// Structure to map between [`Url`] and [`FileId`].
#[derive(Default, Debug, Clone)]
pub struct PathInterner {
    map: IndexSet<VfsPath, BuildHasherDefault<FxHasher>>,
}

impl PathInterner {
    /// Get the id corresponding to [`path`](VfsPath).
    ///
    /// If [`path`](VfsPath) does not exists in `self`, returns [`None`].
    pub fn get(&self, path: &VfsPath) -> Option<FileId> {
        self.map.get_index_of(path).map(|i| FileId(i as u32))
    }

    /// Insert [`path`](VfsPath) in `self`.
    ///
    /// - If [`path`](VfsPath) already exists in `self`, returns its associated [`id`](FileId);
    /// - Else, returns a newly allocated [`id`](FileId).
    pub fn intern(&mut self, path: VfsPath) -> FileId {
        let (id, _added) = self.map.insert_full(path);
        assert!(id < u32::MAX as usize);
        FileId(id as u32)
    }

    /// Returns the path corresponding to [`id`](FileId).
    ///
    /// # Panics
    ///
    /// Panics if [`id`](FileId) does not exists in `self`.
    pub fn lookup(&self, id: FileId) -> &VfsPath {
        self.map.get_index(id.0 as usize).unwrap()
    }
}
