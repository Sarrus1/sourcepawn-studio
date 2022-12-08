use lsp_types::{Hover, HoverParams};

use crate::spitem::get_item_from_position;

use super::FeatureRequest;

pub mod description;

pub fn provide_hover(request: FeatureRequest<HoverParams>) -> Option<Hover> {
    let item = get_item_from_position(
        &request.store,
        request.params.text_document_position_params.position,
        &request
            .params
            .text_document_position_params
            .text_document
            .uri,
    );
    match item {
        Some(item) => item.lock().unwrap().to_hover(&request.params),
        None => None,
    }
}
