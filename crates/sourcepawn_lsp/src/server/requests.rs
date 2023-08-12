use lsp_server::Request;
use lsp_types::request::{
    CallHierarchyIncomingCalls, CallHierarchyOutgoingCalls, CallHierarchyPrepare, Completion,
    DocumentSymbolRequest, GotoDefinition, HoverRequest, References, Rename, ResolveCompletionItem,
    SemanticTokensFullRequest, SignatureHelpRequest,
};

use crate::Server;
use crate::{dispatch, lsp_ext};

mod call_hierarchy;
mod completion;
mod definition;
mod document_symbol;
mod hover;
mod preprocessed_document;
mod projects_graphviz;
mod reference;
mod rename;
mod semantic_tokens;
mod signature_help;

impl Server {
    pub(super) fn handle_request(&mut self, request: Request) -> anyhow::Result<()> {
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
            .on::<lsp_ext::PreprocessedDocument, _>(|id, params| {
                self.preprocessed_document(id, params)
            })?
            .on::<lsp_ext::ProjectsGraphviz, _>(|id, params| self.projects_graphviz(id, params))?
            .default()
        {
            self.connection.sender.send(response.into())?;
        }

        Ok(())
    }
}
