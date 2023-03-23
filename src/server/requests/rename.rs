use crate::utils;
use std::sync::Arc;

use lsp_server::RequestId;
use lsp_types::RenameParams;

use crate::{providers, Server};

impl Server {
    pub(super) fn rename(&mut self, id: RequestId, mut params: RenameParams) -> anyhow::Result<()> {
        utils::normalize_uri(&mut params.text_document_position.text_document.uri);
        let uri = Arc::new(params.text_document_position.text_document.uri.clone());
        self.read_unscanned_document(uri.clone())?;

        self.handle_feature_request(id, params, uri, providers::rename::provide_rename)?;

        Ok(())
    }
}
