use base_db::FilePosition;
use lsp_types::LocationLink;

use crate::RootDatabase;

pub(crate) fn goto_definition(db: &RootDatabase, pos: FilePosition) -> Option<Vec<LocationLink>> {
    None
}
