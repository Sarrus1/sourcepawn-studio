use lsp_types::{Hover, HoverParams};

use crate::store::Store;

pub mod description;

pub fn provide_hover(store: &Store, params: HoverParams) -> Option<Hover> {
    let items = &store.get_items_from_position(
        params.text_document_position_params.position,
        &params.text_document_position_params.text_document.uri,
    );
    if items.is_empty() {
        return None;
    }
    let hover = items[0].read().unwrap().to_hover(&params);

    hover
}
