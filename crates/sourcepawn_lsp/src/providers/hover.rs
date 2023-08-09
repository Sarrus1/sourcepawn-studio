use lsp_types::{Hover, HoverParams};
use store::Store;

pub fn provide_hover(store: &Store, params: HoverParams) -> Option<Hover> {
    let items = &store.get_items_from_position(
        params.text_document_position_params.position,
        &params.text_document_position_params.text_document.uri,
    );
    if items.is_empty() {
        return None;
    }
    let hover = items[0].read().to_hover(&params);

    hover
}
