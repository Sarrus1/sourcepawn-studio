use crate::utils;
use std::sync::Arc;

use lsp_server::RequestId;
use lsp_types::DocumentSymbolParams;

use crate::{providers, Server};

impl Server {
    pub(super) fn document_symbol(
        &mut self,
        id: RequestId,
        mut params: DocumentSymbolParams,
    ) -> anyhow::Result<()> {
        utils::normalize_uri(&mut params.text_document.uri);
        let uri = Arc::new(params.text_document.uri.clone());
        self.read_unscanned_document(uri.clone())?;

        self.handle_feature_request(
            id,
            params,
            uri,
            providers::document_symbol::provide_document_symbol,
        )?;
        Ok(())
    }
}
