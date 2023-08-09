use crate::{lsp_ext::PreprocessedDocumentParams, utils};

use anyhow::bail;
use lsp_server::RequestId;

use crate::Server;

impl Server {
    pub(super) fn preprocessed_document(
        &mut self,
        id: RequestId,
        params: PreprocessedDocumentParams,
    ) -> anyhow::Result<()> {
        let Some(mut text_document) = params.text_document else { bail!("No TextDocument passed to command");};
        utils::normalize_uri(&mut text_document.uri);
        if let Some(document) = self.store.read().documents.get(&text_document.uri) {
            let text = document.preprocessed_text.clone();
            self.run_query(id, move |_store| text);

            return Ok(());
        }

        bail!("No document found for URI {:?}", text_document.uri);
    }
}
