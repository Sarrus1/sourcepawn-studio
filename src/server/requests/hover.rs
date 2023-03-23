use crate::utils;
use std::sync::Arc;

use lsp_server::RequestId;
use lsp_types::HoverParams;

use crate::{providers, Server};

impl Server {
    pub(super) fn hover(&mut self, id: RequestId, mut params: HoverParams) -> anyhow::Result<()> {
        utils::normalize_uri(&mut params.text_document_position_params.text_document.uri);
        let uri = Arc::new(
            params
                .text_document_position_params
                .text_document
                .uri
                .clone(),
        );
        self.read_unscanned_document(uri.clone())?;

        self.handle_feature_request(id, params, uri, providers::hover::provide_hover)?;
        Ok(())
    }
}
