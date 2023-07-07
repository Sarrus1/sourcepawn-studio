use crate::utils;
use std::sync::Arc;

use lsp_server::RequestId;
use lsp_types::{CompletionItem, CompletionParams, Url};

use crate::{providers, Server};

impl Server {
    pub(super) fn completion(
        &mut self,
        id: RequestId,
        mut params: CompletionParams,
    ) -> anyhow::Result<()> {
        utils::normalize_uri(&mut params.text_document_position.text_document.uri);
        let uri = Arc::new(params.text_document_position.text_document.uri.clone());
        let _ = self.read_unscanned_document(uri.clone());

        self.handle_feature_request(id, params, uri, providers::completion::provide_completions)?;

        Ok(())
    }

    pub(super) fn resolve_completion_item(
        &mut self,
        id: RequestId,
        params: CompletionItem,
    ) -> anyhow::Result<()> {
        self.handle_feature_request(
            id,
            params,
            Arc::new(Url::parse("https://example.com").unwrap()), // TODO: Better alternative to this.
            providers::completion::resolve_completion_item,
        )?;

        Ok(())
    }
}
