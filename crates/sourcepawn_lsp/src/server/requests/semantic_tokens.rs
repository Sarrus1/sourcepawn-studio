use lsp_server::RequestId;
use lsp_types::SemanticTokensParams;
use std::sync::Arc;
use store::normalize_uri;

use crate::{providers, Server};

impl Server {
    pub(super) fn semantic_tokens(
        &mut self,
        id: RequestId,
        mut params: SemanticTokensParams,
    ) -> anyhow::Result<()> {
        normalize_uri(&mut params.text_document.uri);
        let uri = Arc::new(params.text_document.uri.clone());

        self.initialize_project_resolution(&uri);
        let _ = self.read_unscanned_document(uri);

        self.run_query(id, move |store| {
            providers::semantic_tokens::provide_semantic_tokens(store, params)
        });

        Ok(())
    }
}
