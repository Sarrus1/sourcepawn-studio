use lsp_server::RequestId;
use lsp_types::GotoDefinitionParams;
use std::sync::Arc;
use store::normalize_uri;

use crate::{providers, Server};

impl Server {
    pub(super) fn definition(
        &mut self,
        id: RequestId,
        mut params: GotoDefinitionParams,
    ) -> anyhow::Result<()> {
        normalize_uri(&mut params.text_document_position_params.text_document.uri);
        let uri = Arc::new(
            params
                .text_document_position_params
                .text_document
                .uri
                .clone(),
        );
        let _ = self.read_unscanned_document(uri);

        self.run_query(id, move |store| {
            providers::definition::provide_definition(store, params)
        });

        Ok(())
    }
}
