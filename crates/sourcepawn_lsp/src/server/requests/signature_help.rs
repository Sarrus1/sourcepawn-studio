use crate::utils;
use std::sync::Arc;

use lsp_server::RequestId;
use lsp_types::SignatureHelpParams;

use crate::{providers, Server};

impl Server {
    pub(super) fn signature_help(
        &mut self,
        id: RequestId,
        mut params: SignatureHelpParams,
    ) -> anyhow::Result<()> {
        utils::normalize_uri(&mut params.text_document_position_params.text_document.uri);
        let uri = Arc::new(
            params
                .text_document_position_params
                .text_document
                .uri
                .clone(),
        );
        let _ = self.read_unscanned_document(uri.clone());

        self.handle_feature_request(
            id,
            params,
            uri,
            providers::signature_help::provide_signature_help,
        )?;

        Ok(())
    }
}
