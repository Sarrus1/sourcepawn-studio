use std::sync::Arc;

use crate::{dispatch, providers::FeatureRequest};

use lsp_server::{Request, RequestId};
use lsp_types::{
    request::{
        Completion, DocumentSymbolRequest, GotoDefinition, HoverRequest, References, Rename,
        SemanticTokensFullRequest, SignatureHelpRequest,
    },
    Url,
};
use serde::Serialize;

use crate::Server;

mod completion;
mod definition;
mod document_symbol;
mod hover;
mod reference;
mod rename;
mod semantic_tokens;
mod signature_help;

impl Server {
    pub(super) fn handle_request(&mut self, request: Request) -> anyhow::Result<()> {
        if self.connection.handle_shutdown(&request)? {
            return Ok(());
        }
        if let Some(response) = dispatch::RequestDispatcher::new(request)
            .on::<Completion, _>(|id, params| self.completion(id, params))?
            .on::<HoverRequest, _>(|id, params| self.hover(id, params))?
            .on::<GotoDefinition, _>(|id, params| self.definition(id, params))?
            .on::<SemanticTokensFullRequest, _>(|id, params| self.semantic_tokens(id, params))?
            .on::<SignatureHelpRequest, _>(|id, params| self.signature_help(id, params))?
            .on::<References, _>(|id, params| self.reference(id, params))?
            .on::<DocumentSymbolRequest, _>(|id, params| self.document_symbol(id, params))?
            .on::<Rename, _>(|id, params| self.rename(id, params))?
            .default()
        {
            self.connection.sender.send(response.into())?;
        }

        Ok(())
    }

    pub(super) fn handle_feature_request<P, R, H>(
        &self,
        id: RequestId,
        params: P,
        uri: Arc<Url>,
        handler: H,
    ) -> anyhow::Result<()>
    where
        P: Send + 'static,
        R: Serialize,
        H: FnOnce(FeatureRequest<P>) -> R + Send + 'static,
    {
        self.spawn(move |server| {
            let request = server.feature_request(uri, params);
            if request.store.iter().next().is_none() {
                let code = lsp_server::ErrorCode::InvalidRequest as i32;
                let message = "unknown document".to_string();
                let response = lsp_server::Response::new_err(id, code, message);
                server.connection.sender.send(response.into()).unwrap();
            } else {
                let result = handler(request);
                server
                    .connection
                    .sender
                    .send(lsp_server::Response::new_ok(id, result).into())
                    .unwrap();
            }
        });

        Ok(())
    }
}
