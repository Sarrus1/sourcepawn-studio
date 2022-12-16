use lsp_types::{Hover, HoverParams};

use crate::spitem::get_items_from_position;

use super::FeatureRequest;

pub mod description;

pub fn provide_hover(request: FeatureRequest<HoverParams>) -> Option<Hover> {
    let items = get_items_from_position(
        &request.store,
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
    let hover = items[0].lock().unwrap().to_hover(&request.params);

    hover
}
