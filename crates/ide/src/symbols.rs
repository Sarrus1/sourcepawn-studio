use hir::Semantics;
use ide_db::{RootDatabase, Symbols, SymbolsBuilder};
use vfs::FileId;

pub(crate) fn symbols(db: &RootDatabase, file_id: FileId) -> Option<Symbols> {
    let sema = &Semantics::new(db);
    let tree = sema.parse(file_id);
    let preprocessing_results = sema.preprocess_file(file_id);
    let source = preprocessing_results.preprocessed_text();

    SymbolsBuilder::new(preprocessing_results.source_map(), &tree, &source)
        .build()
        .into()
}
