use lsp_server::RequestId;
use lsp_types::{
    CallHierarchyIncomingCallsParams, CallHierarchyOutgoingCallsParams, CallHierarchyPrepareParams,
};
use std::sync::Arc;
use store::normalize_uri;

use crate::{providers, Server};

impl Server {
    pub(super) fn call_hierarchy_prepare(
        &mut self,
        id: RequestId,
        mut params: CallHierarchyPrepareParams,
    ) -> anyhow::Result<()> {
        normalize_uri(&mut params.text_document_position_params.text_document.uri);
        let uri = Arc::new(
            params
                .text_document_position_params
                .text_document
                .uri
                .clone(),
        );

        let _ = self.read_unscanned_document(uri);
        self.run_query(id, move |store| {
            providers::call_hierarchy::prepare(store, params)
        });

        Ok(())
    }

    pub(super) fn call_hierarchy_outgoing(
        &mut self,
        id: RequestId,
        mut params: CallHierarchyOutgoingCallsParams,
    ) -> anyhow::Result<()> {
        normalize_uri(&mut params.item.uri);

        self.run_query(id, move |store| {
            providers::call_hierarchy::outgoing(store, params)
        });

        Ok(())
    }

    pub(super) fn call_hierarchy_incoming(
        &mut self,
        id: RequestId,
        mut params: CallHierarchyIncomingCallsParams,
    ) -> anyhow::Result<()> {
        normalize_uri(&mut params.item.uri);

        self.run_query(id, move |store| {
            providers::call_hierarchy::incoming(store, params)
        });

        Ok(())
    }
}
