use crate::{
    capabilities::ClientCapabilitiesExt, client::LspClient, lsp_ext, options::Options,
    providers::FeatureRequest, store::Store,
};
use std::sync::Arc;

use crossbeam_channel::Sender;
use lsp_server::Connection;
use lsp_types::{
    notification::ShowMessage, request::WorkspaceConfiguration, ConfigurationItem,
    ConfigurationParams, MessageType, ShowMessageParams, Url,
};

use super::InternalMessage;

#[derive(Clone)]
pub(super) struct ServerFork {
    pub(super) connection: Arc<Connection>,
    pub(super) internal_tx: Sender<InternalMessage>,
    pub(super) client: LspClient,
    pub(super) store: Store,
}

impl ServerFork {
    pub fn pull_config(&self) -> anyhow::Result<()> {
        if !self
            .store
            .environment
            .client_capabilities
            .has_pull_configuration_support()
        {
            return Ok(());
        }

        let params = ConfigurationParams {
            items: vec![ConfigurationItem {
                section: Some("SourcePawnLanguageServer".to_string()),
                scope_uri: None,
            }],
        };
        match self.client.send_request::<WorkspaceConfiguration>(params) {
            Ok(mut json) => {
                eprintln!("Received config {:?}", json);
                let value = json.pop().expect("invalid configuration request");
                let options = self.parse_options(value)?;
                self.internal_tx
                    .send(InternalMessage::SetOptions(Arc::new(options)))
                    .unwrap();
            }
            Err(why) => {
                eprintln!("Retrieving configuration failed: {}", why);
            }
        };

        Ok(())
    }

    pub fn parse_options(&self, value: serde_json::Value) -> anyhow::Result<Options> {
        let options: Option<Options> = match serde_json::from_value(value) {
            Ok(new_options) => new_options,
            Err(why) => {
                self.client.send_notification::<ShowMessage>(
                    ShowMessageParams {
                        message: format!(
                            "The SourcePawnLanguageServer configuration is invalid; using the default settings instead.\nDetails: {why}"
                        ),
                        typ: MessageType::WARNING,
                    },
                )?;

                None
            }
        };

        Ok(options.unwrap_or_default())
    }

    pub fn feature_request<P>(&self, uri: Arc<Url>, params: P) -> FeatureRequest<P> {
        FeatureRequest {
            params,
            store: self.store.clone(),
            uri,
        }
    }

    pub(crate) fn send_spcomp_status(&self, quiescent: bool) -> anyhow::Result<()> {
        self.client
            .send_notification::<lsp_ext::SpcompStatusNotification>(
                lsp_ext::SpcompStatusParams { quiescent },
            )?;
        Ok(())
    }
}
