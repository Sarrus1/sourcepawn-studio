use std::{fmt, sync::Arc};

use base_db::{FileTextQuery, Tree};
use deepsize::DeepSizeOf;
use fxhash::FxHashMap;
use hir_def::db::{
    AstIdMapQuery, BodyWithSourceMapQuery, ExprScopesQuery, FileDefMapQuery, InferQuery, ParseQuery,
};
use ide_db::RootDatabase;
use preprocessor::{
    db::{PreprocessFileInnerDataQuery, PreprocessFileInnerParamsQuery, PreprocessingParams},
    Macro, PreprocessingResult,
};
use profile::Bytes;
use salsa::{
    debug::{DebugQueryTable, TableEntry},
    Query, QueryTable,
};
use smol_str::SmolStr;
use stdx::{
    format_to,
    hashable_hash_map::{HashableHashMap, HashableHashSet},
};
use vfs::FileId;

// Feature: Status
//
// Shows internal statistic about memory usage of sourcepawn-studio.
//
// |===
// | Editor  | Action Name
//
// | VS Code | **sourcepawn-studio: Status**
// |===
pub(crate) fn status(db: &RootDatabase, _file_id: Option<FileId>) -> String {
    let mut buf = String::new();

    format_to!(buf, "{}\n", collect_query(FileTextQuery.in_db(db)));
    format_to!(buf, "{}\n", collect_query(ParseQuery.in_db(db)));
    format_to!(
        buf,
        "{}\n",
        collect_query(PreprocessFileInnerDataQuery.in_db(db))
    );
    format_to!(
        buf,
        "{}\n",
        collect_query(PreprocessFileInnerParamsQuery.in_db(db))
    );

    format_to!(buf, "\nDebug info:\n");
    format_to!(
        buf,
        "{} ast id maps\n",
        collect_query_count(AstIdMapQuery.in_db(db))
    );
    format_to!(
        buf,
        "{} file def maps\n",
        collect_query_count(FileDefMapQuery.in_db(db))
    );
    format_to!(
        buf,
        "{} body with sourcemap\n",
        collect_query_count(BodyWithSourceMapQuery.in_db(db))
    );
    format_to!(
        buf,
        "{} expr scopes\n",
        collect_query_count(ExprScopesQuery.in_db(db))
    );
    format_to!(buf, "{} infer\n", collect_query_count(InferQuery.in_db(db)));

    buf.trim().to_string()
}

fn collect_query<'q, Q>(table: QueryTable<'q, Q>) -> <Q as QueryCollect>::Collector
where
    QueryTable<'q, Q>: DebugQueryTable,
    Q: QueryCollect,
    <Q as Query>::Storage: 'q,
    <Q as QueryCollect>::Collector: StatCollect<
        <QueryTable<'q, Q> as DebugQueryTable>::Key,
        <QueryTable<'q, Q> as DebugQueryTable>::Value,
    >,
{
    struct StatCollectorWrapper<C>(C);
    impl<C: StatCollect<K, V>, K, V> FromIterator<TableEntry<K, V>> for StatCollectorWrapper<C> {
        fn from_iter<T>(iter: T) -> StatCollectorWrapper<C>
        where
            T: IntoIterator<Item = TableEntry<K, V>>,
        {
            let mut res = C::default();
            for entry in iter {
                res.collect_entry(entry.key, entry.value);
            }
            StatCollectorWrapper(res)
        }
    }
    table
        .entries::<StatCollectorWrapper<<Q as QueryCollect>::Collector>>()
        .0
}

fn collect_query_count<'q, Q>(table: QueryTable<'q, Q>) -> usize
where
    QueryTable<'q, Q>: DebugQueryTable,
    Q: Query,
    <Q as Query>::Storage: 'q,
{
    struct EntryCounter(usize);
    impl<K, V> FromIterator<TableEntry<K, V>> for EntryCounter {
        fn from_iter<T>(iter: T) -> EntryCounter
        where
            T: IntoIterator<Item = TableEntry<K, V>>,
        {
            EntryCounter(iter.into_iter().count())
        }
    }
    table.entries::<EntryCounter>().0
}

trait QueryCollect: Query {
    type Collector;
}

impl QueryCollect for ParseQuery {
    type Collector = SyntaxTreeStats;
}

impl QueryCollect for FileTextQuery {
    type Collector = FilesStats;
}

impl QueryCollect for PreprocessFileInnerDataQuery {
    type Collector = PreprocessDataStats;
}

impl QueryCollect for PreprocessFileInnerParamsQuery {
    type Collector = PreprocessParamsStats;
}

trait StatCollect<K, V>: Default {
    fn collect_entry(&mut self, key: K, value: Option<V>);
}

#[derive(Default)]
struct FilesStats {
    total: usize,
    size: Bytes,
}

impl fmt::Display for FilesStats {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{} of files", self.size)
    }
}

impl StatCollect<FileId, Arc<str>> for FilesStats {
    fn collect_entry(&mut self, _: FileId, value: Option<Arc<str>>) {
        self.total += 1;
        self.size += value.unwrap().len();
    }
}

#[derive(Default)]
pub(crate) struct SyntaxTreeStats {
    total: usize,
    pub(crate) retained: usize,
}

impl fmt::Display for SyntaxTreeStats {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{} trees, {} preserved", self.total, self.retained,)
    }
}

impl StatCollect<FileId, Tree> for SyntaxTreeStats {
    fn collect_entry(&mut self, _: FileId, value: Option<Tree>) {
        self.total += 1;
        self.retained += value.is_some() as usize;
    }
}

#[derive(Default)]
pub(crate) struct PreprocessDataStats {
    total: usize,
    macros: usize,
    macros_size: Bytes,
    offsets: usize,
    pub(crate) retained: usize,
}

impl fmt::Display for PreprocessDataStats {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "{} preprocessed file data, {} preserved, {} macros ({}), {} offsets",
            self.total, self.retained, self.macros, self.macros_size, self.offsets
        )
    }
}

impl StatCollect<(FileId, Arc<PreprocessingParams>), Arc<PreprocessingResult>>
    for PreprocessDataStats
{
    fn collect_entry(
        &mut self,
        _: (FileId, Arc<PreprocessingParams>),
        value: Option<Arc<PreprocessingResult>>,
    ) {
        self.total += 1;
        self.macros += value.as_ref().map(|it| it.macros().len()).unwrap_or(0);
        self.macros_size += value
            .as_ref()
            .map(|it| {
                it.macros()
                    .values()
                    .map(|it| it.deep_size_of())
                    .sum::<usize>()
            })
            .unwrap_or(0);
        self.offsets += value
            .as_ref()
            .map(|it| it.source_map().vec_len())
            .unwrap_or(0);
        self.retained += value.is_some() as usize;
    }
}

#[derive(Default)]
pub(crate) struct PreprocessParamsStats {
    total: usize,
    being_preprocessed: usize,
    macros: usize,
    macros_size: Bytes,
    pub(crate) retained: usize,
}

impl fmt::Display for PreprocessParamsStats {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "{} preprocessed file params, {} preserved, {} being preprocessed, {} macros ({})",
            self.total, self.retained, self.being_preprocessed, self.macros, self.macros_size
        )
    }
}

impl
    StatCollect<
        (
            FileId,
            HashableHashMap<SmolStr, Arc<Macro>>,
            HashableHashSet<FileId>,
        ),
        Arc<FxHashMap<FileId, Arc<PreprocessingParams>>>,
    > for PreprocessParamsStats
{
    fn collect_entry(
        &mut self,
        key: (
            FileId,
            HashableHashMap<SmolStr, Arc<Macro>>,
            HashableHashSet<FileId>,
        ),
        value: Option<Arc<FxHashMap<FileId, Arc<PreprocessingParams>>>>,
    ) {
        self.total += 1;
        self.being_preprocessed += key.2.len();
        self.macros += key.1.len();
        if key.1.is_empty() {
            self.macros_size += 0;
        } else {
            self.macros_size += (key.1.values().map(|it| it.deep_size_of()).sum::<usize>()
                / key.1.len())
                * key.1.capacity();
        }
        self.retained += value.is_some() as usize;
    }
}
