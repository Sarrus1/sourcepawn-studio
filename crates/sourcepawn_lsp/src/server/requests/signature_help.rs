use lsp_server::RequestId;
use lsp_types::SignatureHelpParams;
use std::sync::Arc;
use store::normalize_uri;

use crate::{providers, Server};

impl Server {
    pub(super) fn signature_help(
        &mut self,
        id: RequestId,
        mut params: SignatureHelpParams,
    ) -> anyhow::Result<()> {
        normalize_uri(&mut params.text_document_position_params.text_document.uri);
        let uri = Arc::new(
            params
                .text_document_position_params
                .text_document
                .uri
                .clone(),
        );

        self.initialize_project_resolution(&uri);
        let _ = self.read_unscanned_document(uri);

        self.run_query(id, move |store| {
            providers::signature_help::provide_signature_help(store, params)
        });

        Ok(())
    }
}
