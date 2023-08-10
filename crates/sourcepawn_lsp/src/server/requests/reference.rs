use lsp_server::RequestId;
use lsp_types::ReferenceParams;
use std::sync::Arc;
use store::normalize_uri;

use crate::{providers, Server};

impl Server {
    pub(super) fn reference(
        &mut self,
        id: RequestId,
        mut params: ReferenceParams,
    ) -> anyhow::Result<()> {
        normalize_uri(&mut params.text_document_position.text_document.uri);
        let uri = Arc::new(params.text_document_position.text_document.uri.clone());
        let _ = self.read_unscanned_document(uri);

        self.run_query(id, move |store| {
            providers::reference::provide_reference(store, params)
        });

        Ok(())
    }
}
