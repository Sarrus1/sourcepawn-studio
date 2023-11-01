use crossbeam::channel::{Receiver, Sender};
use fxhash::FxHashMap;
use linter::spcomp::SPCompDiagnostic;
use lsp_server::{Connection, ErrorCode, Message, RequestId};
use lsp_types::{
    notification::ShowMessage, request::WorkspaceConfiguration, CallHierarchyServerCapability,
    ClientCapabilities, ClientInfo, CompletionOptions, CompletionOptionsCompletionItem,
    ConfigurationItem, ConfigurationParams, HoverProviderCapability, InitializeParams,
    InitializeResult, MessageType, OneOf, SemanticTokenModifier, SemanticTokenType,
    SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions,
    SemanticTokensServerCapabilities, ServerCapabilities, ServerInfo, ShowMessageParams,
    SignatureHelpOptions, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
    WorkDoneProgressOptions,
};
use parking_lot::RwLock;
use serde::Serialize;
use std::sync::Arc;
use store::{options::Options, Store};
use threadpool::ThreadPool;

use crate::{capabilities::ClientCapabilitiesExt, client::LspClient, lsp_ext};

mod diagnostics;
mod files;
mod notifications;
mod progress;
mod requests;

#[derive(Debug)]
enum InternalMessage {
    SetOptions(Arc<Options>),
    FileEvent(notify::Event),
    Diagnostics(FxHashMap<Url, Vec<SPCompDiagnostic>>),
}

pub struct Server {
    connection: Arc<Connection>,
    pub store: Arc<RwLock<Store>>,
    internal_tx: Sender<InternalMessage>,
    internal_rx: Receiver<InternalMessage>,
    client: LspClient,
    client_capabilities: Arc<ClientCapabilities>,
    client_info: Option<Arc<ClientInfo>>,
    pool: ThreadPool,
    config_pulled: bool,
    indexing: bool,
    amxxpawn_mode: bool,
}

impl Server {
    pub fn new(connection: Connection, amxxpawn_mode: bool) -> Self {
        let client = LspClient::new(connection.sender.clone());
        let (internal_tx, internal_rx) = crossbeam::channel::unbounded();
        Self {
            connection: Arc::new(connection),
            client,
            internal_rx,
            internal_tx,
            store: Arc::new(RwLock::new(Store::new(amxxpawn_mode))),
            client_capabilities: Default::default(),
            client_info: Default::default(),
            pool: threadpool::Builder::new().build(),
            config_pulled: false,
            indexing: false,
            amxxpawn_mode,
        }
    }

    fn run_query<R, Q>(&self, id: RequestId, query: Q)
    where
        R: Serialize,
        Q: FnOnce(&Store) -> R + Send + 'static,
    {
        let client = self.client.clone();
        let store = Arc::clone(&self.store);
        self.pool.execute(move || {
            let response = lsp_server::Response::new_ok(id, query(&store.read()));
            client.send_response(response).unwrap();
        });
    }

    #[allow(unused)]
    fn run_fallible<R, Q>(&self, id: RequestId, query: Q)
    where
        R: Serialize,
        Q: FnOnce() -> anyhow::Result<R> + Send + 'static,
    {
        let client = self.client.clone();
        self.pool.execute(move || match query() {
            Ok(result) => {
                let response = lsp_server::Response::new_ok(id, result);
                client.send_response(response).unwrap();
            }
            Err(why) => {
                client
                    .send_error(id, ErrorCode::InternalError, why.to_string())
                    .unwrap();
            }
        });
    }

    pub fn pull_config(&self) {
        if !self.client_capabilities.has_pull_configuration_support() {
            log::trace!("Client does not have pull configuration support.");
            return;
        }

        let params = ConfigurationParams {
            items: vec![ConfigurationItem {
                section: Some(
                    if self.amxxpawn_mode {
                        "AMXXPawnLanguageServer"
                    } else {
                        "SourcePawnLanguageServer"
                    }
                    .to_string(),
                ),
                scope_uri: None,
            }],
        };
        let client = self.client.clone();
        let sender = self.internal_tx.clone();
        self.pool.execute(move || {
            match client.send_request::<WorkspaceConfiguration>(params) {
                Ok(mut json) => {
                    log::info!("Received config {:#?}", json);
                    let options = client
                        .parse_options(json.pop().expect("invalid configuration request"))
                        .unwrap();
                    sender
                        .send(InternalMessage::SetOptions(Arc::new(options)))
                        .unwrap();
                }
                Err(why) => {
                    log::error!("Retrieving configuration failed: {}", why);
                }
            };
        });
    }

    /// Resolve the references in a project if they have not been resolved yet. Will return early if the project
    /// has been resolved at least once.
    ///
    /// This should be called before every feature request.
    ///
    /// # Arguments
    /// * `uri` - [Url] of a file in the project.
    fn initialize_project_resolution(&mut self, uri: &Url) {
        log::trace!("Resolving project {:?}", uri);
        let main_id = self.store.write().resolve_project_references(uri);
        if let Some(main_id) = main_id {
            let main_path_uri = self.store.read().path_interner.lookup(main_id).clone();
            self.reload_project_diagnostics(main_path_uri);
        }
        log::trace!("Done resolving project {:?}", uri);
    }

    fn initialize(&mut self) -> anyhow::Result<()> {
        let (id, params) = self.connection.initialize_start()?;
        let params: InitializeParams = serde_json::from_value(params)?;

        let capabilities = ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(
                TextDocumentSyncKind::INCREMENTAL,
            )),
            completion_provider: Some(CompletionOptions {
                trigger_characters: Some(vec![
                    "<".to_string(),
                    '"'.to_string(),
                    "'".to_string(),
                    "/".to_string(),
                    "\\".to_string(),
                    ".".to_string(),
                    ":".to_string(),
                    " ".to_string(),
                    "$".to_string(),
                    "*".to_string(),
                ]),
                resolve_provider: Some(true),
                completion_item: Some(CompletionOptionsCompletionItem {
                    label_details_support: Some(true),
                }),
                ..Default::default()
            }),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            definition_provider: Some(OneOf::Left(true)),
            signature_help_provider: Some(SignatureHelpOptions {
                trigger_characters: Some(vec![",".to_string(), "(".to_string()]),
                retrigger_characters: Some(vec![",".to_string(), "(".to_string()]),
                ..Default::default()
            }),
            references_provider: Some(OneOf::Left(true)),
            document_symbol_provider: Some(OneOf::Left(true)),
            rename_provider: Some(OneOf::Left(true)),
            semantic_tokens_provider: Some(
                SemanticTokensServerCapabilities::SemanticTokensOptions(SemanticTokensOptions {
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None,
                    },
                    legend: SemanticTokensLegend {
                        token_types: vec![
                            SemanticTokenType::VARIABLE,
                            SemanticTokenType::ENUM_MEMBER,
                            SemanticTokenType::FUNCTION,
                            SemanticTokenType::CLASS,
                            SemanticTokenType::METHOD,
                            SemanticTokenType::MACRO,
                            SemanticTokenType::PROPERTY,
                            SemanticTokenType::STRUCT,
                            SemanticTokenType::ENUM,
                        ],
                        token_modifiers: vec![
                            SemanticTokenModifier::READONLY,
                            SemanticTokenModifier::DECLARATION,
                            SemanticTokenModifier::DEPRECATED,
                            SemanticTokenModifier::MODIFICATION,
                        ],
                    },
                    range: Some(false),
                    full: Some(SemanticTokensFullOptions::Delta { delta: Some(false) }),
                }),
            ),
            call_hierarchy_provider: Some(CallHierarchyServerCapability::Simple(true)),
            ..Default::default()
        };
        let result = InitializeResult {
            capabilities,
            server_info: Some(ServerInfo {
                name: "sourcepawn-lsp".to_owned(),
                version: Some(env!("CARGO_PKG_VERSION").to_owned()),
            }),
        };
        self.connection
            .initialize_finish(id, serde_json::to_value(result)?)?;

        self.client_capabilities = Arc::new(params.capabilities);
        self.client_info = params.client_info.map(Arc::new);
        self.store.write().environment.root_uri = params.root_uri;

        self.pull_config();

        self.store.write().folders = params
            .workspace_folders
            .unwrap_or_default()
            .iter()
            .filter_map(|folder| folder.uri.to_file_path().ok())
            .collect();

        let _ = self.send_status(lsp_ext::ServerStatusParams {
            health: crate::lsp_ext::Health::Ok,
            quiescent: !self.indexing,
            message: None,
        });
        log::trace!("Server is initialized.");

        Ok(())
    }

    fn send_status(&self, status: lsp_ext::ServerStatusParams) -> anyhow::Result<()> {
        self.client
            .send_notification::<lsp_ext::ServerStatusNotification>(status)?;
        Ok(())
    }

    fn process_messages(&mut self) -> anyhow::Result<()> {
        loop {
            crossbeam::channel::select! {
                recv(&self.connection.receiver) -> msg => {
                        match msg? {
                            Message::Request(request) => {
                                log::trace!("Received request {:#?}", request);
                                if self.connection.handle_shutdown(&request)? {
                                    log::trace!("Handled shutdown request.");
                                    return Ok(());
                                }
                                if let Err(error) = self.handle_request(request) {
                                    let _ = self.send_status(lsp_ext::ServerStatusParams {
                                        health: crate::lsp_ext::Health::Error,
                                        quiescent: !self.indexing,
                                        message: Some(error.to_string()),
                                    });
                                }
                            }
                            Message::Response(resp) => {
                                if let Err(error) = self.client.recv_response(resp) {
                                    let _ = self.send_status(lsp_ext::ServerStatusParams {
                                        health: crate::lsp_ext::Health::Error,
                                        quiescent: !self.indexing,
                                        message: Some(error.to_string()),
                                    });
                                }
                            }
                            Message::Notification(notification) => {
                                if let Err(error) = self.handle_notification(notification) {
                                    let _ = self.send_status(lsp_ext::ServerStatusParams {
                                        health: crate::lsp_ext::Health::Error,
                                        quiescent: !self.indexing,
                                        message: Some(error.to_string()),
                                    });
                                }
                            }
                        }
                    }
                    recv(&self.internal_rx) -> msg => {
                        match msg? {
                            InternalMessage::SetOptions(options) => {
                                self.config_pulled = true;
                                self.store.write().environment.options = options;
                                self.register_file_watching()?;
                                if let Err(err) = self.reparse_all() {
                                    let _ = self.client
                                        .send_notification::<ShowMessage>(ShowMessageParams {
                                            message: format!("Failed to reparse all files: {:?}", err),
                                            typ: MessageType::ERROR,
                                        });
                                }
                            }
                            InternalMessage::FileEvent(event) => {
                                self.handle_file_event(event);
                            }
                            InternalMessage::Diagnostics(diagnostics) => {
                                self.store.write().diagnostics.ingest_spcomp_diagnostics(diagnostics);
                                self.publish_diagnostics();
                            }
                        }
                }
            }
        }
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        log::debug!(
            "sourcepawn-lsp will use a maximum of {} threads.",
            self.pool.max_count()
        );
        self.initialize()?;
        self.process_messages()?;
        Ok(())
    }
}
