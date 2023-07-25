use lsp_types::{SignatureHelp, SignatureHelpParams, SignatureInformation};

use self::signature_attributes::SignatureAttributes;

use super::FeatureRequest;

mod signature_attributes;

/// Build a [SignatureHelp](lsp_types::SignatureHelp) from a [SignatureHelpParams](lsp_types::SignatureHelpParams).
///
/// # Arguments
///
/// * `request` - Signature Help request object [FeatureRequest<SignatureHelpParams>].
pub fn provide_signature_help(
    request: FeatureRequest<SignatureHelpParams>,
) -> Option<SignatureHelp> {
    let uri = request
        .params
        .text_document_position_params
        .text_document
        .uri;
    let document = request.store.documents.get(&uri)?;
    let signature_attributes = SignatureAttributes::get_signature_attributes(
        document,
        request.params.text_document_position_params.position,
    )?;

    let items = &request
        .store
        .get_items_from_position(signature_attributes.position, uri);
    let mut signatures: Vec<SignatureInformation> = Vec::new();
    for item in items {
        let signature_help = item
            .read()
            .unwrap()
            .to_signature_help(signature_attributes.parameter_count);
        if let Some(signature_help) = signature_help {
            signatures.push(signature_help);
        }
    }

    Some(SignatureHelp {
        signatures,
        active_parameter: None,
        active_signature: None,
    })
}
