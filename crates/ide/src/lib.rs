//! base_db defines basic database traits. The concrete DB is defined by ide.

mod completion;
mod events;
mod goto_definition;
mod hover;
mod markup;
mod prime_caches;
mod references;
mod rename;
mod signature_help;
mod status;
mod syntax_highlighting;

use std::{panic::AssertUnwindSafe, sync::Arc};

use base_db::{
    Change, FileExtension, FilePosition, FileRange, Graph, SourceDatabase, SourceDatabaseExt, Tree,
};
use fxhash::FxHashMap;
use hir::DefResolution;
use hir_def::{print_item_tree, DefDatabase};
use hover::HoverResult;
use ide_db::{RootDatabase, SourceChange};
use itertools::Itertools;
use lsp_types::Url;
use paths::AbsPathBuf;
use preprocessor::db::PreprocDatabase;
use salsa::{Cancelled, ParallelDatabase};
use serde_json::Value;
use vfs::FileId;

pub use completion::{CompletionItem, CompletionKind};
pub use goto_definition::NavigationTarget;
pub use hover::{HoverAction, HoverConfig, HoverDocFormat, HoverGotoTypeData};
pub use ide_db::Cancellable;
pub use ide_diagnostics::{Diagnostic, DiagnosticsConfig, Severity};
pub use line_index::{LineCol, LineIndex, WideEncoding, WideLineCol};
pub use markup::Markup;
pub use prime_caches::ParallelPrimeCachesProgress;
pub use signature_help::SignatureHelp;
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
    pub fn new(lru_capacity: Option<usize>) -> AnalysisHost {
        AnalysisHost {
            db: RootDatabase::new(lru_capacity),
        }
    }

    pub fn update_lru_capacity(&mut self, lru_capacity: Option<usize>) {
        self.db.update_parse_query_lru_capacity(lru_capacity);
    }

    pub fn update_lru_capacities(&mut self, lru_capacities: &FxHashMap<Box<str>, usize>) {
        self.db.update_lru_capacities(lru_capacities);
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

    /// Get all the root files of the projects that depend on the file.
    pub fn projects_for_file(&self, file_id: FileId) -> Cancellable<Vec<FileId>> {
        self.with_db(|db| {
            let graph = db.graph();
            let subgraphs = graph.find_subgraphs();
            subgraphs
                .iter()
                .filter_map(|subgraph| {
                    if subgraph.contains_file(file_id) {
                        Some(subgraph.root.file_id)
                    } else {
                        None
                    }
                })
                .collect_vec()
        })
    }

    /// Debug info about the current state of the analysis.
    pub fn status(&self, file_id: Option<FileId>) -> Cancellable<String> {
        self.with_db(|db| status::status(db, file_id))
    }

    pub fn parallel_prime_caches<F1, F2>(
        &self,
        num_worker_threads: u8,
        cb: F1,
        file_id_to_name: F2,
    ) -> Cancellable<()>
    where
        F1: Fn(ParallelPrimeCachesProgress) + Sync + std::panic::UnwindSafe,
        F2: Fn(FileId) -> Option<String> + Sync + std::panic::UnwindSafe,
    {
        self.with_db(move |db| {
            prime_caches::parallel_prime_caches(db, num_worker_threads, &cb, file_id_to_name)
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

    /// Returns the references for the symbol at `position`.
    pub fn references(&self, pos: FilePosition) -> Cancellable<Option<Vec<FileRange>>> {
        self.with_db(|db| references::references(db, pos))
    }

    /// Returns the source change to rename the symbol at `position` to `new_name`.
    pub fn rename(&self, fpos: FilePosition, new_name: &str) -> Cancellable<Option<SourceChange>> {
        self.with_db(|db| rename::rename(db, fpos, new_name))
    }

    /// Returns the hover information at `position`.
    pub fn hover(
        &self,
        pos: FilePosition,
        config: &HoverConfig,
        file_id_to_url: AssertUnwindSafe<&dyn Fn(FileId) -> Option<String>>,
        events_game_name: Option<&str>,
    ) -> Cancellable<Option<RangeInfo<HoverResult>>> {
        self.with_db(|db| hover::hover(db, pos, config, file_id_to_url, events_game_name))
    }

    /// Returns the hover information at `position`.
    pub fn signature_help(&self, pos: FilePosition) -> Cancellable<Option<SignatureHelp>> {
        self.with_db(|db| signature_help::signature_help(db, pos))
    }

    /// Returns the completions at `position`.
    pub fn completions(
        &self,
        position: FilePosition,
        trigger_character: Option<char>,
        include_directories: Vec<AbsPathBuf>,
        file_id_to_url: AssertUnwindSafe<&dyn Fn(FileId) -> Url>,
        events_game_name: Option<&str>,
    ) -> Cancellable<Option<Vec<CompletionItem>>> {
        self.with_db(|db| {
            completion::completions(
                db,
                position,
                trigger_character,
                include_directories,
                file_id_to_url,
                events_game_name,
            )
            .map(Into::into)
        })
    }

    pub fn resolve_completion(
        &self,
        data: Value,
        item: lsp_types::CompletionItem,
    ) -> Cancellable<Option<lsp_types::CompletionItem>> {
        let data: DefResolution =
            serde_json::from_value(data).expect("failed to deserialize completion data");
        self.with_db(|db| completion::resolve_completion(db, data, item))
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
