//! base_db defines basic database traits. The concrete DB is defined by ide.

mod call_item;
mod documentation;
mod source_change;
mod symbols;

use std::{fmt, mem::ManuallyDrop, sync::Arc};

use base_db::{
    Change, FileLoader, FileLoaderDelegate, SourceDatabaseExt, SourceDatabaseExtStorage,
    SourceDatabaseStorage, Upcast,
};
use fxhash::FxHashMap;
use hir::{db::HirDatabase, FunctionType};
use hir_def::DefDatabase;
use line_index::LineIndex;
use salsa::{Cancelled, Durability};
use vfs::FileId;

pub use call_item::{CallItem, IncomingCallItem, OutgoingCallItem};
pub use documentation::Documentation;
pub use source_change::SourceChange;
pub use symbols::{Symbol, SymbolId, Symbols, SymbolsBuilder};

pub type Cancellable<T> = Result<T, Cancelled>;

pub type FxIndexSet<T> = indexmap::IndexSet<T, std::hash::BuildHasherDefault<fxhash::FxHasher>>;
pub type FxIndexMap<K, V> =
    indexmap::IndexMap<K, V, std::hash::BuildHasherDefault<fxhash::FxHasher>>;

#[salsa::database(
    SourceDatabaseExtStorage,
    SourceDatabaseStorage,
    hir_def::db::InternDatabaseStorage,
    hir_def::db::DefDatabaseStorage,
    preprocessor::db::PreprocDatabaseStorage,
    LineIndexDatabaseStorage,
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
        RootDatabase::new(None)
    }
}

impl RootDatabase {
    pub fn new(lru_capacity: Option<usize>) -> RootDatabase {
        let mut db = RootDatabase {
            storage: ManuallyDrop::new(salsa::Storage::default()),
        };
        db.set_known_files_with_durability(Default::default(), Durability::HIGH);
        db.set_source_roots_with_durability(Default::default(), Durability::HIGH);
        db.update_parse_query_lru_capacity(lru_capacity);
        db
    }

    pub fn update_parse_query_lru_capacity(&mut self, lru_capacity: Option<usize>) {
        let lru_capacity = lru_capacity.unwrap_or(base_db::DEFAULT_PARSE_LRU_CAP);
        hir_def::db::ParseQuery
            .in_db_mut(self)
            .set_lru_capacity(lru_capacity);
        preprocessor::db::PreprocessFileQuery
            .in_db_mut(self)
            .set_lru_capacity(lru_capacity);
        preprocessor::db::PreprocessedTextQuery
            .in_db_mut(self)
            .set_lru_capacity(lru_capacity);
        preprocessor::db::PreprocessFileInnerDataQuery
            .in_db_mut(self)
            .set_lru_capacity(lru_capacity);
        preprocessor::db::PreprocessFileInnerParamsQuery
            .in_db_mut(self)
            .set_lru_capacity(lru_capacity);
    }

    pub fn update_lru_capacities(&mut self, lru_capacities: &FxHashMap<Box<str>, usize>) {
        base_db::GraphQuery.in_db_mut(self).set_lru_capacity(
            lru_capacities
                .get(stringify!(GraphQuery))
                .copied()
                .unwrap_or(base_db::DEFAULT_PARSE_LRU_CAP),
        );
        base_db::ProjetSubgraphQuery
            .in_db_mut(self)
            .set_lru_capacity(
                lru_capacities
                    .get(stringify!(ProjectSubgraphQuery))
                    .copied()
                    .unwrap_or(base_db::DEFAULT_PARSE_LRU_CAP),
            );

        macro_rules! update_lru_capacity_per_query {
            ($( $module:ident :: $query:ident )*) => {$(
                if let Some(&cap) = lru_capacities.get(stringify!($query)) {
                    $module::$query.in_db_mut(self).set_lru_capacity(cap);
                }
            )*}
        }
        // FIXME: Implement this
        update_lru_capacity_per_query![
            // HIR Def
            hir_def::FileItemTreeQuery
            hir_def::BlockDefMapQuery
            hir_def::BlockItemTreeQuery
            hir_def::BodyQuery
            hir_def::FileDefMapQuery
            hir_def::FileItemTreeQuery
        ];
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

#[salsa::query_group(LineIndexDatabaseStorage)]
pub trait LineIndexDatabase: base_db::SourceDatabase {
    fn line_index(&self, file_id: FileId) -> Arc<LineIndex>;
}

fn line_index(db: &dyn LineIndexDatabase, file_id: FileId) -> Arc<LineIndex> {
    let text = db.file_text(file_id);
    Arc::new(LineIndex::new(&text))
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SymbolKind {
    #[default]
    Macro,
    Function,
    Forward,
    Native,
    Constructor,
    Destructor,
    Typedef,
    Typeset,
    Functag,
    Funcenum,
    Method,
    EnumStruct,
    Field,
    Methodmap,
    Property,
    Struct,
    Enum,
    Variant,
    Global,
    Local,
}

impl From<FunctionType> for SymbolKind {
    fn from(ft: FunctionType) -> Self {
        match ft {
            FunctionType::Function => SymbolKind::Function,
            FunctionType::Constructor => SymbolKind::Constructor,
            FunctionType::Method => SymbolKind::Method,
            FunctionType::Getter => SymbolKind::Method,
            FunctionType::Setter => SymbolKind::Method,
            FunctionType::Destructor => SymbolKind::Destructor,
        }
    }
}
