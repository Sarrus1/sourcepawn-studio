use lsp_types::{Location, ReferenceParams};
use store::Store;

pub fn provide_reference(store: &Store, params: ReferenceParams) -> Option<Vec<Location>> {
    let items = &store.get_items_from_position(
        params.text_document_position.position,
        &params.text_document_position.text_document.uri,
    );
    let mut locations = vec![];
    for item in items {
        let item = item.read();
        let references = item.references();
        if let Some(references) = references {
            locations.extend(references.clone());
        }
    }

    Some(
        locations
            .iter()
            .map(|reference| {
                let uri = store.path_interner.lookup(reference.file_id);
                Location {
                    uri: uri.clone(),
                    range: reference.v_range,
                }
            })
            .collect(),
    )
}
