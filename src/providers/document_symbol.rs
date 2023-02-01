use lsp_types::{DocumentSymbol, DocumentSymbolParams};

use super::FeatureRequest;

pub fn provide_document_symbol(
    request: FeatureRequest<DocumentSymbolParams>,
) -> Option<Vec<DocumentSymbol>> {
    let uri = request.params.text_document.uri;
    let document = request.store.documents.get(&uri)?;
    let mut symbols: Vec<DocumentSymbol> = vec![];
    for item in document.sp_items.clone() {
        let symbol = item.lock().unwrap().to_document_symbol();
        if let Some(symbol) = symbol {
            symbols.push(symbol);
        }
    }

    Some(symbols)
}
