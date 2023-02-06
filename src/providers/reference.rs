use lsp_types::{Location, ReferenceParams};

use crate::spitem::get_items_from_position;

use super::FeatureRequest;

/// Build a vector of [Locations](lsp_types::Location) from a [ReferenceParams](lsp_types::ReferenceParams).
///
/// # Arguments
///
/// * `request` - Reference request object [FeatureRequest<ReferenceParams>].
pub fn provide_reference(request: FeatureRequest<ReferenceParams>) -> Option<Vec<Location>> {
    let items = get_items_from_position(
        &request.store,
        request.params.text_document_position.position,
        request
            .params
            .text_document_position
            .text_document
            .uri
            .clone(),
    );
    let mut locations = vec![];
    for item in items {
        let item = item.read().unwrap();
        let references = item.references();
        if let Some(references) = references {
            locations.extend(references.clone());
        }
    }

    Some(locations.iter().map(|loc| loc.to_lsp_location()).collect())
}
