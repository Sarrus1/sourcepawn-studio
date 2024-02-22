//! base_db defines basic database traits. The concrete DB is defined by ide.

mod goto_definition;
mod hover;
mod markup;
mod syntax_highlighting;

use std::sync::Arc;

use base_db::{
    Change, FileExtension, FilePosition, FileRange, Graph, SourceDatabase, SourceDatabaseExt, Tree,
};
use hir_def::{print_item_tree, DefDatabase};
use hover::HoverResult;
use ide_db::RootDatabase;
use preprocessor::db::PreprocDatabase;
use salsa::{Cancelled, ParallelDatabase};
use vfs::FileId;

pub use goto_definition::NavigationTarget;
pub use hover::{HoverConfig, HoverDocFormat};
pub use ide_db::Cancellable;
pub use ide_diagnostics::{Diagnostic, DiagnosticsConfig, Severity};
pub use line_index::{LineCol, LineIndex, WideEncoding, WideLineCol};
pub use markup::Markup;
pub use syntax_highlighting::{Highlight, HlMod, HlMods, HlRange, HlTag};

/// Info associated with a [`range`](lsp_types::Range).
#[derive(Debug)]
pub struct RangeInfo<T> {
    pub range: lsp_types::Range,
    pub info: T,
}

impl<T> RangeInfo<T> {
    pub fn new(range: lsp_types::Range, info: T) -> RangeInfo<T> {
        RangeInfo { range, info }
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

    pub fn raw_database(&self) -> &RootDatabase {
        &self.db
    }

    pub fn set_known_files(&mut self, files: Vec<(FileId, FileExtension)>) {
        self.db.set_known_files(files);
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

    pub fn graph(&self) -> Cancellable<Arc<Graph>> {
        self.with_db(|db| db.graph())
    }

    /// Gets the preprocessed text of the file.
    pub fn preprocessed_text(&self, file_id: FileId) -> Cancellable<Arc<str>> {
        self.with_db(|db| db.preprocessed_text(file_id))
    }

    /// Gets the [`String`] representation of the item tree of the file.
    pub fn pretty_item_tree(&self, file_id: FileId) -> Cancellable<String> {
        self.with_db(|db| {
            let item_tree = db.file_item_tree(file_id);
            print_item_tree(db, &item_tree)
        })
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

    /// Computes the set of diagnostics for the given file.
    pub fn diagnostics(
        &self,
        config: &DiagnosticsConfig,
        file_id: FileId,
    ) -> Cancellable<Vec<Diagnostic>> {
        self.with_db(|db| ide_diagnostics::diagnostics(db, config, file_id))
    }

    /// Returns the definitions from the symbol at `position`.
    pub fn goto_definition(
        &self,
        pos: FilePosition,
    ) -> Cancellable<Option<RangeInfo<Vec<NavigationTarget>>>> {
        self.with_db(|db| goto_definition::goto_definition(db, pos))
    }

    /// Returns the hover information at `position`.
    pub fn hover(
        &self,
        pos: FilePosition,
        config: &HoverConfig,
    ) -> Cancellable<Option<RangeInfo<HoverResult>>> {
        self.with_db(|db| hover::hover(db, pos, config))
    }

    /// Returns the highlighted ranges for the file.
    pub fn highlight(&self, file_id: FileId) -> Cancellable<Vec<syntax_highlighting::HlRange>> {
        self.with_db(|db| syntax_highlighting::highlight(db, file_id, None))
    }

    /// Computes syntax highlighting for the given file range.
    pub fn highlight_range(&self, frange: FileRange) -> Cancellable<Vec<HlRange>> {
        self.with_db(|db| syntax_highlighting::highlight(db, frange.file_id, Some(frange.range)))
    }
}
