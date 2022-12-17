use crate::{dispatch, options::Options, providers::FeatureRequest, store::Store, utils};
use std::{path::PathBuf, sync::Arc, time::Instant};

use crossbeam_channel::{Receiver, Sender};
use lsp_server::{Connection, Message, RequestId};
use lsp_types::{
    notification::{DidChangeTextDocument, DidOpenTextDocument, ShowMessage},
    request::{Completion, GotoDefinition, HoverRequest, WorkspaceConfiguration},
    CompletionOptions, CompletionParams, ConfigurationItem, ConfigurationParams,
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, GotoDefinitionParams, HoverParams,
    HoverProviderCapability, InitializeParams, MessageType, OneOf, ServerCapabilities,
    ShowMessageParams, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
};
use serde::Serialize;
use threadpool::ThreadPool;
use tree_sitter::Parser;

use crate::client::LspClient;
use crate::providers;

#[derive(Debug)]
enum InternalMessage {
    SetOptions(Arc<Options>),
}

#[derive(Clone)]
struct ServerFork {
    connection: Arc<Connection>,
    internal_tx: Sender<InternalMessage>,
    client: LspClient,
    store: Store,
}

impl ServerFork {
    pub fn pull_config(&self) -> anyhow::Result<()> {
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
}

impl Server {
    pub fn new(connection: Connection, current_dir: PathBuf) -> Self {
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
            store: Store::new(current_dir),
            pool: threadpool::Builder::new().build(),
            parser,
            config_pulled: false,
        }
    }

    fn initialize(&mut self) -> anyhow::Result<()> {
        let server_capabilities = serde_json::to_value(&ServerCapabilities {
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
                ]),
                ..Default::default()
            }),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            definition_provider: Some(OneOf::Left(true)),
            ..Default::default()
        })
        .unwrap();
        let initialization_params = self.connection.initialize(server_capabilities)?;
        let params: InitializeParams = serde_json::from_value(initialization_params).unwrap();
        self.store.environment.client_capabilities = Arc::new(params.capabilities);
        self.store.environment.client_info = params.client_info.map(Arc::new);

        self.spawn(move |server| {
            let _ = server.pull_config();
        });
        let base_path = params.workspace_folders.unwrap()[0]
            .uri
            .to_file_path()
            .unwrap();
        self.store.find_documents(&base_path);
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

    fn did_open(&mut self, mut params: DidOpenTextDocumentParams) -> anyhow::Result<()> {
        if !self.config_pulled {
            return Ok(());
        }
        utils::normalize_uri(&mut params.text_document.uri);
        let uri = Arc::new(params.text_document.uri);

        // Don't parse the document if it has already been opened.
        // GoToDefinition request will trigger a new parse.
        let document = self.store.documents.get(&uri);
        if let Some(document) = document {
            if document.parsed {
                return Ok(());
            }
        }
        let text = params.text_document.text;
        self.store
            .handle_open_document(uri, text, &mut self.parser)
            .expect("Couldn't parse file");

        Ok(())
    }

    fn did_change(&mut self, mut params: DidChangeTextDocumentParams) -> anyhow::Result<()> {
        utils::normalize_uri(&mut params.text_document.uri);

        let uri = Arc::new(params.text_document.uri.clone());

        match self.store.get(&uri) {
            Some(old_document) => {
                let mut text = old_document.text().to_string();
                utils::apply_document_edit(&mut text, params.content_changes);
                self.store
                    .handle_open_document(uri, text, &mut self.parser)?;
            }
            None => match uri.to_file_path() {
                Ok(path) => {
                    self.store.load(path, &mut self.parser)?;
                }
                Err(_) => return Ok(()),
            },
        };

        Ok(())
    }

    fn reparse_all(&mut self) -> anyhow::Result<()> {
        self.store.parse_directories();
        let main_uri = self.store.environment.options.get_main_path_uri();
        if main_uri.is_none() {
            // Send a warning for a potential invalid main path here.
            let mut uris: Vec<Url> = vec![];
            for uri in self.store.documents.keys() {
                uris.push(uri.as_ref().clone());
            }
            for uri in uris.iter() {
                let document = self.store.get(uri);
                if let Some(document) = document {
                    self.store
                        .handle_open_document(document.uri, document.text, &mut self.parser)
                        .unwrap();
                }
            }
            eprintln!("Stop");
            return Ok(());
        }
        let main_uri = main_uri.unwrap();
        let document = self
            .store
            .get(&main_uri)
            .expect("Main Path does not exist.");
        let now = Instant::now();
        self.store
            .handle_open_document(document.uri, document.text, &mut self.parser)
            .expect("Couldn't parse file");
        eprintln!("Reparsed all the files in {:.2?}", now.elapsed());

        Ok(())
    }

    fn completion(&self, id: RequestId, mut params: CompletionParams) -> anyhow::Result<()> {
        utils::normalize_uri(&mut params.text_document_position.text_document.uri);
        let uri = Arc::new(params.text_document_position.text_document.uri.clone());
        self.handle_feature_request(id, params, uri, providers::completion::provide_completions)?;
        Ok(())
    }

    fn hover(&self, id: RequestId, mut params: HoverParams) -> anyhow::Result<()> {
        utils::normalize_uri(&mut params.text_document_position_params.text_document.uri);
        let uri = Arc::new(
            params
                .text_document_position_params
                .text_document
                .uri
                .clone(),
        );
        self.handle_feature_request(id, params, uri, providers::hover::provide_hover)?;
        Ok(())
    }

    fn definition(&self, id: RequestId, mut params: GotoDefinitionParams) -> anyhow::Result<()> {
        utils::normalize_uri(&mut params.text_document_position_params.text_document.uri);
        let uri = Arc::new(
            params
                .text_document_position_params
                .text_document
                .uri
                .clone(),
        );
        self.handle_feature_request(id, params, uri, providers::definition::provide_definition)?;
        Ok(())
    }

    fn handle_feature_request<P, R, H>(
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

    fn process_messages(&mut self) -> anyhow::Result<()> {
        loop {
            crossbeam_channel::select! {
                recv(&self.connection.receiver) -> msg => {
                        // eprintln!("got msg: {:?}", msg);
                        match msg? {
                            Message::Request(request) => {
                                if self.connection.handle_shutdown(&request)? {
                                    return Ok(());
                                }
                                if let Some(response) = dispatch::RequestDispatcher::new(request)
                                .on::<Completion, _>(|id, params| self.completion(id, params))?
                                .on::<HoverRequest, _>(|id, params| self.hover(id, params))?
                                .on::<GotoDefinition, _>(|id, params| self.definition(id, params))?
                                .default()
                                {
                                    self.connection.sender.send(response.into())?;
                                }
                            }
                            Message::Response(resp) => {
                                self.client.recv_response(resp)?;
                            }
                            Message::Notification(notification) => {
                                dispatch::NotificationDispatcher::new(notification)
                                .on::<DidOpenTextDocument, _>(|params| self.did_open(params))?
                                .on::<DidChangeTextDocument, _>(|params| self.did_change(params))?
                                .default();
                                }
                        }
                    }
                    recv(&self.internal_rx) -> msg => {
                        match msg? {
                            InternalMessage::SetOptions(options) => {
                                self.config_pulled = true;
                                self.store.environment.options = options;
                                self.reparse_all().expect("Failed to reparse all files.");
                            }
                        }
                }
            }
        }
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        self.initialize()?;
        self.process_messages()?;
        Ok(())
    }
}
