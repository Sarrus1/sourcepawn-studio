//! base_db defines basic database traits. The concrete DB is defined by ide.

use std::{fmt, mem::ManuallyDrop, sync::Arc};

use base_db::{
    Change, FileLoader, FileLoaderDelegate, SourceDatabaseExtStorage, SourceDatabaseStorage, Upcast,
};
use hir::db::HirDatabase;
use hir_def::DefDatabase;
use salsa::Cancelled;
use vfs::FileId;

pub type Cancellable<T> = Result<T, Cancelled>;

#[salsa::database(
    SourceDatabaseExtStorage,
    SourceDatabaseStorage,
    hir_def::db::InternDatabaseStorage,
    hir_def::db::DefDatabaseStorage,
    hir::db::HirDatabaseStorage
)]
pub struct RootDatabase {
    // We use `ManuallyDrop` here because every codegen unit that contains a
    // `&RootDatabase -> &dyn OtherDatabase` cast will instantiate its drop glue in the vtable,
    // which duplicates `Weak::drop` and `Arc::drop` tens of thousands of times, which makes
    // compile times of all `ide_*` and downstream crates suffer greatly.
    storage: ManuallyDrop<salsa::Storage<RootDatabase>>,
}

impl Drop for RootDatabase {
    fn drop(&mut self) {
        unsafe { ManuallyDrop::drop(&mut self.storage) };
    }
}

impl fmt::Debug for RootDatabase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RootDatabase").finish()
    }
}

impl Upcast<dyn DefDatabase> for RootDatabase {
    #[inline]
    fn upcast(&self) -> &(dyn DefDatabase + 'static) {
        self
    }
}

impl Upcast<dyn HirDatabase> for RootDatabase {
    #[inline]
    fn upcast(&self) -> &(dyn HirDatabase + 'static) {
        self
    }
}

impl FileLoader for RootDatabase {
    fn file_text(&self, file_id: FileId) -> Arc<str> {
        FileLoaderDelegate(self).file_text(file_id)
    }
    fn known_files(&self) -> Vec<(FileId, base_db::FileExtension)> {
        FileLoaderDelegate(self).known_files()
    }
    fn resolve_path(&self, uri: vfs::AnchoredPath<'_>) -> Option<FileId> {
        FileLoaderDelegate(self).resolve_path(uri)
    }
    fn resolve_path_relative_to_roots(&self, path: &str) -> Option<FileId> {
        FileLoaderDelegate(self).resolve_path_relative_to_roots(path)
    }
}

impl salsa::Database for RootDatabase {}

impl Default for RootDatabase {
    fn default() -> Self {
        RootDatabase::new()
    }
}

impl RootDatabase {
    pub fn new() -> Self {
        RootDatabase {
            storage: ManuallyDrop::new(salsa::Storage::default()),
        }
    }

    pub fn apply_change(&mut self, change: Change) {
        change.apply(self);
    }
}

impl salsa::ParallelDatabase for RootDatabase {
    fn snapshot(&self) -> salsa::Snapshot<RootDatabase> {
        salsa::Snapshot::new(RootDatabase {
            storage: ManuallyDrop::new(self.storage.snapshot()),
        })
    }
}
