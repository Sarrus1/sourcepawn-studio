//! # Virtual File System
//!
//! **Note:** This is heavily inspired from rust-analyzer.
//!
//! VFS stores all files read by sourcepawn-lsp. Reading file contents from VFS
//! always returns the same contents, unless VFS was explicitly modified with
//! [`set_file_contents`]. All changes to VFS are logged, and can be retrieved via
//! the [`take_changes`] method.
//!
//! Files in VFS are identified with [`FileId`]s -- interned paths.
//!
//! [`set_file_contents`]: Vfs::set_file_contents
//! [`take_changes`]: Vfs::take_changes

use std::{
    fmt::{self},
    mem,
};

use lsp_types::Url;
use path_interner::PathInterner;

mod anchored_path;
mod file_id;
mod file_set;
mod path_interner;

pub use {
    anchored_path::{AnchoredUrl, AnchoredUrlBuf},
    file_id::FileId,
    file_set::{FileSet, FileSetConfig},
};

/// Storage for all files read by rust-analyzer.
///
/// For more information see the [crate-level](crate) documentation.
#[derive(Default)]
pub struct Vfs {
    interner: PathInterner,
    data: Vec<Option<Vec<u8>>>,
    changes: Vec<ChangedFile>,
}

/// Changed file in the [`Vfs`].
#[derive(Debug)]
pub struct ChangedFile {
    /// Id of the changed file
    pub file_id: FileId,
    /// Kind of change
    pub change_kind: ChangeKind,
}

impl ChangedFile {
    /// Returns `true` if the change is not [`Delete`](ChangeKind::Delete).
    pub fn exists(&self) -> bool {
        self.change_kind != ChangeKind::Delete
    }

    /// Returns `true` if the change is [`Create`](ChangeKind::Create) or
    /// [`Delete`](ChangeKind::Delete).
    pub fn is_created_or_deleted(&self) -> bool {
        matches!(self.change_kind, ChangeKind::Create | ChangeKind::Delete)
    }
}

/// Kind of [file change](ChangedFile).
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum ChangeKind {
    /// The file was (re-)created
    Create,
    /// The file was modified
    Modify,
    /// The file was deleted
    Delete,
}

impl Vfs {
    /// Id of the given path if it exists in the `Vfs` and is not deleted.
    pub fn file_id(&self, uri: &Url) -> Option<FileId> {
        self.interner.get(uri).filter(|&it| self.get(it).is_some())
    }

    /// File path corresponding to the given `file_id`.
    ///
    /// # Panics
    ///
    /// Panics if the id is not present in the `Vfs`.
    pub fn file_path(&self, file_id: FileId) -> Url {
        self.interner.lookup(file_id).clone()
    }

    /// File content corresponding to the given `file_id`.
    ///
    /// # Panics
    ///
    /// Panics if the id is not present in the `Vfs`, or if the corresponding file is
    /// deleted.
    pub fn file_contents(&self, file_id: FileId) -> &[u8] {
        self.get(file_id).as_deref().unwrap()
    }

    /// Returns the overall memory usage for the stored files.
    pub fn memory_usage(&self) -> usize {
        self.data.iter().flatten().map(|d| d.capacity()).sum()
    }

    /// Returns an iterator over the stored ids and their corresponding paths.
    ///
    /// This will skip deleted files.
    pub fn iter(&self) -> impl Iterator<Item = (FileId, &Url)> + '_ {
        (0..self.data.len())
            .map(|it| FileId(it as u32))
            .filter(move |&file_id| self.get(file_id).is_some())
            .map(move |file_id| {
                let path = self.interner.lookup(file_id);
                (file_id, path)
            })
    }

    /// Update the `path` with the given `contents`. `None` means the file was deleted.
    ///
    /// Returns `true` if the file was modified, and saves the [change](ChangedFile).
    ///
    /// If the path does not currently exists in the `Vfs`, allocates a new
    /// [`FileId`] for it.
    pub fn set_file_contents(&mut self, path: Url, mut contents: Option<Vec<u8>>) -> bool {
        let file_id = self.alloc_file_id(path);
        let change_kind = match (self.get(file_id), &contents) {
            (None, None) => return false,
            (Some(old), Some(new)) if old == new => return false,
            (None, Some(_)) => ChangeKind::Create,
            (Some(_), None) => ChangeKind::Delete,
            (Some(_), Some(_)) => ChangeKind::Modify,
        };
        if let Some(contents) = &mut contents {
            contents.shrink_to_fit();
        }
        *self.get_mut(file_id) = contents;
        self.changes.push(ChangedFile {
            file_id,
            change_kind,
        });
        true
    }

    /// Returns `true` if the `Vfs` contains [changes](ChangedFile).
    pub fn has_changes(&self) -> bool {
        !self.changes.is_empty()
    }

    /// Drain and returns all the changes in the `Vfs`.
    pub fn take_changes(&mut self) -> Vec<ChangedFile> {
        mem::take(&mut self.changes)
    }

    /// Provides a panic-less way to verify file_id validity.
    pub fn exists(&self, file_id: FileId) -> bool {
        self.get(file_id).is_some()
    }

    /// Returns the id associated with `path`
    ///
    /// - If `path` does not exists in the `Vfs`, allocate a new id for it, associated with a
    /// deleted file;
    /// - Else, returns `path`'s id.
    ///
    /// Does not record a change.
    fn alloc_file_id(&mut self, uri: Url) -> FileId {
        let file_id = self.interner.intern(uri);
        let idx = file_id.0 as usize;
        let len = self.data.len().max(idx + 1);
        self.data.resize_with(len, || None);
        file_id
    }

    /// Returns the content associated with the given `file_id`.
    ///
    /// # Panics
    ///
    /// Panics if no file is associated to that id.
    fn get(&self, file_id: FileId) -> &Option<Vec<u8>> {
        &self.data[file_id.0 as usize]
    }

    /// Mutably returns the content associated with the given `file_id`.
    ///
    /// # Panics
    ///
    /// Panics if no file is associated to that id.
    fn get_mut(&mut self, file_id: FileId) -> &mut Option<Vec<u8>> {
        &mut self.data[file_id.0 as usize]
    }
}

impl fmt::Debug for Vfs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Vfs")
            .field("n_files", &self.data.len())
            .finish()
    }
}
