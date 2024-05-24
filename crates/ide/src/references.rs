use base_db::{FilePosition, FileRange};
use hir::Semantics;
use ide_db::RootDatabase;

pub(crate) fn references(db: &RootDatabase, fpos: FilePosition) -> Option<Vec<FileRange>> {
    let sema = &Semantics::new(db);
    let res = sema.find_references_from_pos(fpos)?;

    Some(res.1)
}
