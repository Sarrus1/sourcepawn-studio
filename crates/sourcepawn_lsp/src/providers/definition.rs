use lsp_types::{GotoDefinitionParams, GotoDefinitionResponse};

use crate::store::Store;

pub fn provide_definition(
    store: &Store,
    params: GotoDefinitionParams,
) -> Option<GotoDefinitionResponse> {
    let items = &store.get_items_from_position(
        params.text_document_position_params.position,
        &params.text_document_position_params.text_document.uri,
    );
    if items.is_empty() {
        return None;
    }
    let mut definitions = vec![];
    for item in items.iter() {
        match item.read().to_definition(&params) {
            Some(definition) => definitions.push(definition),
            None => {
                continue;
            }
        }
    }

    Some(GotoDefinitionResponse::Link(definitions))
}
