use crate::{lsp_ext::PreprocessedDocumentParams, providers::FeatureRequest, utils};
use std::sync::Arc;

use anyhow::anyhow;
use lsp_server::RequestId;

use crate::Server;

impl Server {
    pub(super) fn preprocessed_document(
        &mut self,
        id: RequestId,
        params: PreprocessedDocumentParams,
    ) -> anyhow::Result<()> {
        if let Some(mut text_document) = params.text_document.clone() {
            utils::normalize_uri(&mut text_document.uri);
            if let Some(document) = self.store.documents.get(&text_document.uri) {
                let text = document.preprocessed_text.clone();
                self.handle_feature_request(
                    id,
                    params,
                    Arc::new(text_document.uri),
                    |_: FeatureRequest<PreprocessedDocumentParams>| text,
                )?;

                return Ok(());
            }

            return Err(anyhow!("No document found for URI {:?}", text_document.uri));
        }

        Err(anyhow!("No TextDocument passed to command"))
    }
}
