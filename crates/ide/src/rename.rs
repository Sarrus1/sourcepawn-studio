use base_db::FilePosition;
use hir::Semantics;
use ide_db::{RootDatabase, SourceChange};
use lsp_types::TextEdit;

pub(crate) fn rename(
    db: &RootDatabase,
    fpos: FilePosition,
    new_name: &str,
) -> Option<SourceChange> {
    let sema = &Semantics::new(db);
    let refs = sema.find_references_from_pos(fpos)?;
    let mut res = SourceChange::default();
    refs.1.iter().for_each(|it| {
        res.insert(it.file_id, TextEdit::new(it.range, new_name.to_string()));
    });

    res.into()
}
