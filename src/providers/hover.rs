use lsp_types::{Hover, HoverContents, HoverParams};

use crate::spitem::get_item_from_position;

use super::FeatureRequest;

pub mod description;

pub fn provide_hover(request: FeatureRequest<HoverParams>) -> Option<Hover> {
    let item = get_item_from_position(
        &request.store,
        request.params.text_document_position_params.position,
    );
    eprintln!("FOUND {:?}", item);
    if item.is_none() {
        return None;
    }
    let item = item.unwrap();

    Some(Hover {
        contents: HoverContents::Markup(item.documentation().unwrap().description_to_md()),
        range: item.range(),
    })
}
