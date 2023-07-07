use std::sync::Arc;

use crate::{dispatch, providers::FeatureRequest};

use lsp_server::{Request, RequestId};
use lsp_types::{
    request::{
        CallHierarchyIncomingCalls, CallHierarchyOutgoingCalls, CallHierarchyPrepare, Completion,
        DocumentSymbolRequest, GotoDefinition, HoverRequest, References, Rename,
        ResolveCompletionItem, SemanticTokensFullRequest, SignatureHelpRequest,
    },
    Url,
};
use serde::Serialize;

use crate::Server;

mod call_hierarchy;
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
        log::trace!("Received request {:#?}", request);
        if self.connection.handle_shutdown(&request)? {
            log::trace!("Handled shutdown request.");
            return Ok(());
        }
        if let Some(response) = dispatch::RequestDispatcher::new(request)
            .on::<Completion, _>(|id, params| self.completion(id, params))?
            .on::<ResolveCompletionItem, _>(|id, params| self.resolve_completion_item(id, params))?
            .on::<HoverRequest, _>(|id, params| self.hover(id, params))?
            .on::<GotoDefinition, _>(|id, params| self.definition(id, params))?
            .on::<SemanticTokensFullRequest, _>(|id, params| self.semantic_tokens(id, params))?
            .on::<SignatureHelpRequest, _>(|id, params| self.signature_help(id, params))?
            .on::<References, _>(|id, params| self.reference(id, params))?
            .on::<DocumentSymbolRequest, _>(|id, params| self.document_symbol(id, params))?
            .on::<Rename, _>(|id, params| self.rename(id, params))?
            .on::<CallHierarchyOutgoingCalls, _>(|id, params| {
                self.call_hierarchy_outgoing(id, params)
            })?
            .on::<CallHierarchyIncomingCalls, _>(|id, params| {
                self.call_hierarchy_incoming(id, params)
            })?
            .on::<CallHierarchyPrepare, _>(|id, params| self.call_hierarchy_prepare(id, params))?
            .default()
        {
            self.connection.sender.send(response.into())?;
        }
        log::trace!("Handled request.");

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
                match server.connection.sender.send(response.into()) {
                    Ok(_) => {}
                    Err(error) => {
                        log::error!("Failed to send response: {}", error);
                    }
                }
            } else {
                let result = handler(request);
                match server
                    .connection
                    .sender
                    .send(lsp_server::Response::new_ok(id, result).into())
                {
                    Ok(_) => {}
                    Err(error) => {
                        log::error!("Failed to send response: {}", error);
                    }
                }
            }
        });

        Ok(())
    }
}
