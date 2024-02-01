//! Partitions a list of files into disjoint subsets.
//!
//! Files which do not belong to any explicitly configured `FileSet` belong to
//! the default `FileSet`.
use std::{fmt, path::PathBuf};

use fxhash::FxHashMap;
use lsp_types::Url;
use nohash_hasher::IntMap;

use crate::{anchored_path::AnchoredUrl, normalize_uri, FileId, Vfs};

/// A set of [`VfsPath`]s identified by [`FileId`]s.
#[derive(Default, Clone, Eq, PartialEq)]
pub struct FileSet {
    files: FxHashMap<Url, FileId>,
    uris: IntMap<FileId, Url>,
}

impl FileSet {
    /// Returns the number of stored paths.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Get the id of the file corresponding to `path`.
    ///
    /// If either `uri`'s [`anchor`](AnchoredUrl::anchor) or the resolved path is not in
    /// the set, returns [`None`].
    pub fn resolve_path(&self, uri: AnchoredUrl<'_>) -> Option<FileId> {
        // FIXME: Account for case insensitive filesystems.
        let path = PathBuf::from(uri.uri);
        // For absolute paths, we can just canonicalize and look it up.
        if path.is_absolute() {
            return self.files.get(&Url::from_file_path(path).ok()?).copied();
        }
        // Try relative to the anchor.
        let mut base = self.uris[&uri.anchor].clone().to_file_path().ok()?;
        base.pop();
        let path = match dunce::canonicalize(base.join(uri.uri)) {
            Ok(path) => path,
            Err(err) => {
                eprintln!("failed to canonicalize {:?} {:?}", uri, err);
                return None;
            }
        };
        let mut uri = Url::from_file_path(path).ok()?;
        normalize_uri(&mut uri);
        self.files.get(&uri).copied()
    }

    /// Get the id corresponding to `uri` if it exists in the set.
    pub fn file_for_uri(&self, uri: &Url) -> Option<&FileId> {
        self.files.get(uri)
    }

    /// Get the [`Url`] corresponding to `file` if it exists in the set.
    pub fn path_for_file(&self, file: &FileId) -> Option<&Url> {
        self.uris.get(file)
    }

    /// Insert the `file_id, uri` pair into the set.
    ///
    /// # Note
    /// Multiple [`FileId`] can be mapped to the same [`Url`], and vice-versa.
    pub fn insert(&mut self, file_id: FileId, uri: Url) {
        self.files.insert(uri.clone(), file_id);
        self.uris.insert(file_id, uri);
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
    roots: Vec<Url>,
}

impl FileSetConfig {
    pub fn set_roots(&mut self, mut roots: Vec<Url>) {
        roots.iter_mut().for_each(normalize_uri);
        self.roots = roots;
    }

    /// Partition `vfs` into `FileSet`s.
    ///
    /// Creates a new [`FileSet`] for every set of prefixes in `self`.
    pub fn partition(&mut self, vfs: &Vfs) -> Vec<FileSet> {
        self.roots.dedup();
        let mut res = vec![FileSet::default(); self.len()];
        for (file_id, uri) in vfs.iter() {
            for (root, root_uri) in self.roots.iter().enumerate() {
                // FIXME: This breaks for nested roots.
                eprintln!("checking if {:?} starts with {:?}", uri, root_uri);
                if uri.as_str().starts_with(root_uri.as_str()) {
                    res[root].insert(file_id, uri.clone());
                    break;
                }
            }
        }
        res
    }

    /// Number of sets that `self` should partition a [`Vfs`] into.
    fn len(&self) -> usize {
        self.roots.len()
    }
}
