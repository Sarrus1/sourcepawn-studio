use crate::{linter::spcomp::SPCompDiagnostic, lsp_ext, options::Options, store::Store};
use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};
use fxhash::FxHashMap;
use lsp_server::{Connection, Message};
use lsp_types::{
    CallHierarchyServerCapability, CompletionOptions, CompletionOptionsCompletionItem,
    HoverProviderCapability, InitializeParams, InitializeResult, OneOf, SemanticTokenModifier,
    SemanticTokenType, SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions,
    SemanticTokensServerCapabilities, ServerCapabilities, ServerInfo, SignatureHelpOptions,
    TextDocumentSyncCapability, TextDocumentSyncKind, Url, WorkDoneProgressOptions,
};

use threadpool::ThreadPool;
use tree_sitter::Parser;

use crate::client::LspClient;

use self::fork::ServerFork;

mod diagnostics;
mod files;
mod fork;
mod notifications;
mod requests;

#[derive(Debug)]
enum InternalMessage {
    SetOptions(Arc<Options>),
    FileEvent(notify::Event),
    Diagnostics(FxHashMap<Url, Vec<SPCompDiagnostic>>),
}

pub struct Server {
    connection: Arc<Connection>,
    client: LspClient,
    pub store: Store,
    internal_tx: Sender<InternalMessage>,
    internal_rx: Receiver<InternalMessage>,
    pool: ThreadPool,
    parser: Parser,
    config_pulled: bool,
    indexing: bool,
}

impl Server {
    pub fn new(connection: Connection, amxxpawn_mode: bool) -> Self {
        let client = LspClient::new(connection.sender.clone());
        let (internal_tx, internal_rx) = crossbeam_channel::unbounded();
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_sourcepawn::language())
            .expect("Error loading SourcePawn grammar");
        Self {
            connection: Arc::new(connection),
            client,
            internal_rx,
            internal_tx,
            store: Store::new(amxxpawn_mode),
            pool: threadpool::Builder::new().build(),
            parser,
            config_pulled: false,
            indexing: false,
        }
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

        self.store.environment.client_capabilities = Arc::new(params.capabilities);
        self.store.environment.client_info = params.client_info.map(Arc::new);
        self.store.environment.root_uri = params.root_uri;

        self.spawn(move |server| {
            let _ = server.pull_config();
        });
        for folder in params.workspace_folders.unwrap_or_default() {
            self.store
                .find_documents(&folder.uri.to_file_path().unwrap())
        }

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

    fn spawn(&self, job: impl FnOnce(ServerFork) + Send + 'static) {
        let fork = self.fork();
        self.pool.execute(move || job(fork));
    }

    fn fork(&self) -> ServerFork {
        ServerFork {
            connection: self.connection.clone(),
            client: self.client.clone(),
            internal_tx: self.internal_tx.clone(),
            store: self.store.clone(),
        }
    }

    fn process_messages(&mut self) -> anyhow::Result<()> {
        loop {
            crossbeam_channel::select! {
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
                                self.store.environment.options = options;
                                self.register_file_watching()?;
                                self.reparse_all().expect("Failed to reparse all files.");
                            }
                            InternalMessage::FileEvent(event) => {
                                self.handle_file_event(event);
                            }
                            InternalMessage::Diagnostics(diagnostics) => {
                                self.store.ingest_spcomp_diagnostics(diagnostics);
                                self.publish_diagnostics()?;
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
