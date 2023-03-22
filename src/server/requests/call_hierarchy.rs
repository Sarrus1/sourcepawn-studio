use crate::utils;
use std::sync::Arc;

use lsp_server::RequestId;
use lsp_types::{
    CallHierarchyIncomingCallsParams, CallHierarchyOutgoingCallsParams, CallHierarchyPrepareParams,
};

use crate::{providers, Server};

impl Server {
    pub(super) fn call_hierarchy_prepare(
        &mut self,
        id: RequestId,
        mut params: CallHierarchyPrepareParams,
    ) -> anyhow::Result<()> {
        utils::normalize_uri(&mut params.text_document_position_params.text_document.uri);
        let uri = Arc::new(
            params
                .text_document_position_params
                .text_document
                .uri
                .clone(),
        );

        self.read_unscanned_document(uri.clone());

        self.handle_feature_request(id, params, uri, providers::call_hierarchy::prepare)?;

        Ok(())
    }

    pub(super) fn call_hierarchy_outgoing(
        &mut self,
        id: RequestId,
        mut params: CallHierarchyOutgoingCallsParams,
    ) -> anyhow::Result<()> {
        utils::normalize_uri(&mut params.item.uri);
        let uri = Arc::new(params.item.uri.clone());

        self.handle_feature_request(id, params, uri, providers::call_hierarchy::outgoing)?;
        Ok(())
    }

    pub(super) fn call_hierarchy_incoming(
        &mut self,
        id: RequestId,
        mut params: CallHierarchyIncomingCallsParams,
    ) -> anyhow::Result<()> {
        utils::normalize_uri(&mut params.item.uri);
        let uri = Arc::new(params.item.uri.clone());

        self.handle_feature_request(id, params, uri, providers::call_hierarchy::incoming)?;
        Ok(())
    }
}
