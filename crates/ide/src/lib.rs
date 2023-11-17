//! base_db defines basic database traits. The concrete DB is defined by ide.

mod goto_definition;

use std::{fmt, mem::ManuallyDrop, sync::Arc};

use base_db::{
    Change, FileLoader, FileLoaderDelegate, FilePosition, SourceDatabase, SourceDatabaseExtStorage,
    SourceDatabaseStorage, Tree,
};
use salsa::{Cancelled, ParallelDatabase};
use vfs::FileId;

pub use line_index::{LineCol, LineIndex, WideEncoding, WideLineCol};

pub type Cancellable<T> = Result<T, Cancelled>;

#[salsa::database(
    SourceDatabaseExtStorage,
    SourceDatabaseStorage,
    hir_def::db::InternDatabaseStorage,
    hir_def::db::DefDatabaseStorage
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

impl FileLoader for RootDatabase {
    fn file_text(&self, file_id: FileId) -> Arc<str> {
        FileLoaderDelegate(self).file_text(file_id)
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

/// `AnalysisHost` stores the current state of the world.
#[derive(Debug, Default)]
pub struct AnalysisHost {
    db: RootDatabase,
}

impl AnalysisHost {
    pub fn new() -> AnalysisHost {
        AnalysisHost {
            db: RootDatabase::new(),
        }
    }

    /// Returns a snapshot of the current state, which you can query for
    /// semantic information.
    pub fn analysis(&self) -> Analysis {
        Analysis {
            db: self.db.snapshot(),
        }
    }

    /// Applies changes to the current state of the world.
    pub fn apply_change(&mut self, change: Change) {
        self.db.apply_change(change)
    }
}

/// Analysis is a snapshot of a server state at a moment in time. It is the main
/// entry point for asking semantic information about the server. When the server
/// state is advanced using the [`AnalysisHost::apply_change`] method, all
/// existing [`Analysis`] are canceled (most method return [`Err(Canceled)`]).
#[derive(Debug)]
pub struct Analysis {
    db: salsa::Snapshot<RootDatabase>,
}

impl Analysis {
    /// Gets the text of the source file.
    pub fn file_text(&self, file_id: FileId) -> Cancellable<Arc<str>> {
        self.with_db(|db| db.file_text(file_id))
    }

    /// Gets the syntax tree of the file.
    pub fn parse(&self, file_id: FileId) -> Cancellable<Tree> {
        self.with_db(|db| db.parse(file_id))
    }

    /// Performs an operation on the database that may be canceled.
    ///
    /// sourcepawn-lsp needs to be able to answer semantic questions about the
    /// code while the code is being modified. A common problem is that a
    /// long-running query is being calculated when a new change arrives.
    ///
    /// We can't just apply the change immediately: this will cause the pending
    /// query to see inconsistent state (it will observe an absence of
    /// repeatable read). So what we do is we **cancel** all pending queries
    /// before applying the change.
    ///
    /// Salsa implements cancellation by unwinding with a special value and
    /// catching it on the API boundary.
    fn with_db<F, T>(&self, f: F) -> Cancellable<T>
    where
        F: FnOnce(&RootDatabase) -> T + std::panic::UnwindSafe,
    {
        Cancelled::catch(|| f(&self.db))
    }

    /// Returns the definitions from the symbol at `position`.
    pub fn goto_definition(
        &self,
        pos: FilePosition,
    ) -> Cancellable<Option<Vec<lsp_types::LocationLink>>> {
        self.with_db(|db| goto_definition::goto_definition(db, pos))
    }
}
