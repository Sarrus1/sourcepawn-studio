use lsp_types::{GotoDefinitionParams, GotoDefinitionResponse};

use super::FeatureRequest;

pub fn provide_definition(
    request: FeatureRequest<GotoDefinitionParams>,
) -> Option<GotoDefinitionResponse> {
    let items = &request.store.get_items_from_position(
        request.params.text_document_position_params.position,
        request
            .params
            .text_document_position_params
            .text_document
            .uri
            .clone(),
    );
    if items.is_empty() {
        return None;
    }
    let mut definitions = vec![];
    for item in items.iter() {
        match item.read().unwrap().to_definition(&request.params) {
            Some(definition) => definitions.push(definition),
            None => {
                continue;
            }
        }
    }

    Some(GotoDefinitionResponse::Link(definitions))
}
