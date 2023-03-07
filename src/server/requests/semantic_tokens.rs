use crate::utils;
use std::sync::Arc;

use lsp_server::RequestId;
use lsp_types::SemanticTokensParams;

use crate::{providers, Server};

impl Server {
    pub(super) fn semantic_tokens(
        &mut self,
        id: RequestId,
        mut params: SemanticTokensParams,
    ) -> anyhow::Result<()> {
        utils::normalize_uri(&mut params.text_document.uri);
        let uri = Arc::new(params.text_document.uri.clone());
        self.read_unscanned_document(uri.clone());

        self.handle_feature_request(
            id,
            params,
            uri,
            providers::semantic_tokens::provide_semantic_tokens,
        )?;
        Ok(())
    }
}
