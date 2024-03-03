//! Partitions a list of files into disjoint subsets.
//!
//! Files which do not belong to any explicitly configured `FileSet` belong to
//! the default `FileSet`.
use std::{fmt, path::PathBuf};

use fxhash::FxHashMap;
use nohash_hasher::IntMap;
use paths::AbsPathBuf;

use crate::{anchored_path::AnchoredPath, vfs_path::VfsPath, FileId, Vfs};

/// A set of [`VfsPath`]s identified by [`FileId`]s.
#[derive(Default, Clone, Eq, PartialEq)]
pub struct FileSet {
    files: FxHashMap<VfsPath, FileId>,
    uris: IntMap<FileId, VfsPath>,
}

impl FileSet {
    /// Returns the number of stored paths.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Returns `true` if the set contains no paths.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Get the id of the file corresponding to `path`.
    ///
    /// If either `path`'s [`anchor`](AnchoredUrl::anchor) or the resolved path is not in
    /// the set, returns [`None`].
    pub fn resolve_path(&self, path: AnchoredPath<'_>) -> Option<FileId> {
        // FIXME: Account for case insensitive filesystems.
        let abs_path = PathBuf::from(path.path);
        // For absolute paths, we can just canonicalize and look it up.
        if abs_path.is_absolute() {
            let abs_path = AbsPathBuf::try_from(abs_path).ok()?;
            return self.files.get(&VfsPath::from(abs_path)).copied();
        }

        // Try relative to the anchor.
        let mut base = self.uris[&path.anchor].clone();
        base.pop();
        let path = base.join(path.path)?;
        self.files.get(&path).copied()
    }

    pub fn resolve_path_relative_to_root(&self, root: &VfsPath, path: &str) -> Option<FileId> {
        let path = root.join(path)?;
        self.files.get(&path).copied()
    }

    /// Get the id corresponding to `path` if it exists in the set.
    pub fn file_for_path(&self, uri: &VfsPath) -> Option<&FileId> {
        self.files.get(uri)
    }

    /// Get the [`VfsPath`] corresponding to `file` if it exists in the set.
    pub fn path_for_file(&self, file: &FileId) -> Option<&VfsPath> {
        self.uris.get(file)
    }

    /// Insert the `file_id, path` pair into the set.
    ///
    /// # Note
    /// Multiple [`FileId`] can be mapped to the same [`VfsPath`], and vice-versa.
    pub fn insert(&mut self, file_id: FileId, path: VfsPath) {
        self.files.insert(path.clone(), file_id);
        self.uris.insert(file_id, path);
    }

    /// Iterate over this set's ids.
    pub fn iter(&self) -> impl Iterator<Item = FileId> + '_ {
        self.uris.keys().copied()
    }
}

impl fmt::Debug for FileSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileSet")
            .field("n_files", &self.files.len())
            .finish()
    }
}

#[derive(Debug, Default)]
pub struct FileSetConfig {
    /// Roots of the file sets.
    roots: Vec<VfsPath>,
}

impl FileSetConfig {
    pub fn set_roots(&mut self, roots: Vec<VfsPath>) {
        self.roots = roots;
    }

    /// Partition `vfs` into `FileSet`s.
    ///
    /// Creates a new [`FileSet`] for every set of prefixes in `self`.
    pub fn partition(&mut self, vfs: &Vfs) -> Vec<(FileSet, VfsPath)> {
        self.roots.dedup();
        let mut res = Vec::new();
        for root in self.roots.iter() {
            res.push((FileSet::default(), root.clone()));
        }
        for (file_id, path) in vfs.iter() {
            for (root, root_path) in self.roots.iter().enumerate() {
                // FIXME: This breaks for nested roots.
                if path.starts_with(root_path) {
                    res[root].0.insert(file_id, path.clone());
                    break;
                }
            }
        }
        res
    }

    /// Number of sets that `self` should partition a [`Vfs`] into.
    #[allow(unused)]
    fn len(&self) -> usize {
        self.roots.len()
    }
}
