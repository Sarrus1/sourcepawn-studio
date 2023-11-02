use lsp_types::{DocumentSymbol, DocumentSymbolParams};
use store::Store;

pub fn provide_document_symbol(
    store: &Store,
    params: DocumentSymbolParams,
) -> Option<Vec<DocumentSymbol>> {
    let file_id = store.vfs.get(&params.text_document.uri)?;
    let document = store.documents.get(&file_id)?;
    let mut symbols: Vec<DocumentSymbol> = vec![];
    for item in document.sp_items.clone() {
        let symbol = item.read().to_document_symbol();
        if let Some(symbol) = symbol {
            symbols.push(symbol);
        }
    }

    Some(symbols)
}
