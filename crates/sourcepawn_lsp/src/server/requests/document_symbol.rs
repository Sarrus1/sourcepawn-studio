use std::sync::Arc;

use lsp_server::RequestId;
use lsp_types::DocumentSymbolParams;
use store::normalize_uri;

use crate::{providers, Server};

impl Server {
    pub(super) fn document_symbol(
        &mut self,
        id: RequestId,
        mut params: DocumentSymbolParams,
    ) -> anyhow::Result<()> {
        normalize_uri(&mut params.text_document.uri);
        let uri = Arc::new(params.text_document.uri.clone());
        let _ = self.read_unscanned_document(uri);

        self.run_query(id, move |store| {
            providers::document_symbol::provide_document_symbol(store, params)
        });

        Ok(())
    }
}
