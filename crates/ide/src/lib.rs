//! base_db defines basic database traits. The concrete DB is defined by ide.

mod goto_definition;
mod hover;
mod markup;
mod prime_caches;
mod status;
mod syntax_highlighting;

use std::{panic::AssertUnwindSafe, sync::Arc};

use base_db::{
    Change, FileExtension, FilePosition, FileRange, Graph, SourceDatabase, SourceDatabaseExt, Tree,
};
use fxhash::FxHashMap;
use hir_def::{print_item_tree, DefDatabase};
use hover::HoverResult;
use ide_db::RootDatabase;
use itertools::Itertools;
use preprocessor::{db::PreprocDatabase, ArgsMap, Offset};
use salsa::{Cancelled, ParallelDatabase};
use syntax::range_contains_pos;
use vfs::FileId;

pub use goto_definition::NavigationTarget;
pub use hover::{HoverAction, HoverConfig, HoverDocFormat, HoverGotoTypeData};
pub use ide_db::Cancellable;
pub use ide_diagnostics::{Diagnostic, DiagnosticsConfig, Severity};
pub use line_index::{LineCol, LineIndex, WideEncoding, WideLineCol};
pub use markup::Markup;
pub use prime_caches::ParallelPrimeCachesProgress;
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

    /// Returns the hover information at `position`.
    pub fn hover(
        &self,
        pos: FilePosition,
        config: &HoverConfig,
        file_id_to_url: AssertUnwindSafe<&dyn Fn(FileId) -> Option<String>>,
    ) -> Cancellable<Option<RangeInfo<HoverResult>>> {
        self.with_db(|db| hover::hover(db, pos, config, file_id_to_url))
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

/// Convert a position seen by the user to a position seen by the server (preprocessed).
///
/// Will try to look for a mapped range of a macro argument and return the offsetted position.
/// If no mapped range is found, will try to apply the offsets to the position.
///
/// # Arguments
///
/// * `args_map` - The preprocessed arguments map.
/// * `offsets` - The preprocessed offsets.
/// * `pos` - The position to convert.
///
/// # Returns
///
/// The source position range, if a mapped range was found.
fn u_pos_to_s_pos(
    args_map: &ArgsMap,
    offsets: &FxHashMap<u32, Vec<Offset>>,
    pos: &mut lsp_types::Position,
) -> Option<lsp_types::Range> {
    let mut source_u_range = None;

    match args_map.get(&pos.line).and_then(|args| {
        args.iter()
            .find(|(range, _)| range_contains_pos(range, pos))
    }) {
        Some((u_range, s_range)) => {
            *pos = s_range.start;
            source_u_range = Some(*u_range);
        }
        None => {
            if let Some(diff) = offsets.get(&pos.line).map(|offsets| {
                offsets
                    .iter()
                    .filter(|offset| offset.range.end.character <= pos.character)
                    .map(|offset| offset.diff.saturating_sub_unsigned(offset.args_diff))
                    .sum::<i32>()
            }) {
                *pos = lsp_types::Position {
                    line: pos.line,
                    character: pos.character.saturating_add_signed(diff),
                };
            }
        }
    }

    source_u_range
}

/// Convert a range seen by the server to a range seen by the user.
fn s_range_to_u_range(
    offsets: &FxHashMap<u32, Vec<Offset>>,
    mut s_range: lsp_types::Range,
) -> lsp_types::Range {
    if let Some(offsets) = offsets.get(&s_range.start.line) {
        for offset in offsets.iter() {
            if offset.range.start.character < s_range.start.character {
                s_range.start.character = s_range
                    .start
                    .character
                    .saturating_add_signed(-offset.diff.saturating_sub_unsigned(offset.args_diff));
            }
        }
        for offset in offsets.iter() {
            if offset.range.start.character < s_range.end.character {
                s_range.end.character = s_range
                    .end
                    .character
                    .saturating_add_signed(-offset.diff.saturating_sub_unsigned(offset.args_diff));
            }
        }
    }

    s_range
}
