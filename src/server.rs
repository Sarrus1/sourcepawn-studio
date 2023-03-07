use crate::{
    capabilities::ClientCapabilitiesExt, dispatch, lsp_ext, options::Options,
    providers::FeatureRequest, store::Store, utils,
};
use std::{path::PathBuf, sync::Arc, time::Instant};

use crossbeam_channel::{Receiver, Sender};
use lsp_server::{Connection, Message, RequestId};
use lsp_types::{
    notification::{
        DidChangeConfiguration, DidChangeTextDocument, DidOpenTextDocument, ShowMessage,
    },
    request::{
        Completion, DocumentSymbolRequest, GotoDefinition, HoverRequest, References,
        SemanticTokensFullRequest, SignatureHelpRequest, WorkspaceConfiguration,
    },
    CompletionOptions, CompletionParams, ConfigurationItem, ConfigurationParams,
    DidChangeConfigurationParams, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
    DocumentSymbolParams, GotoDefinitionParams, HoverParams, HoverProviderCapability,
    InitializeParams, MessageType, OneOf, ReferenceParams, SemanticTokenModifier,
    SemanticTokenType, SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions,
    SemanticTokensParams, SemanticTokensServerCapabilities, ServerCapabilities, ShowMessageParams,
    SignatureHelpOptions, SignatureHelpParams, TextDocumentSyncCapability, TextDocumentSyncKind,
    Url, WorkDoneProgressOptions,
};
use notify::Watcher;
use serde::Serialize;
use threadpool::ThreadPool;
use tree_sitter::Parser;
use walkdir::WalkDir;

use crate::client::LspClient;
use crate::providers;

#[derive(Debug)]
enum InternalMessage {
    SetOptions(Arc<Options>),
    FileEvent(notify::Event),
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
            indexing: false,
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
            signature_help_provider: Some(SignatureHelpOptions {
                trigger_characters: Some(vec![",".to_string(), "(".to_string()]),
                retrigger_characters: Some(vec![",".to_string(), "(".to_string()]),
                ..Default::default()
            }),
            references_provider: Some(OneOf::Left(true)),
            document_symbol_provider: Some(OneOf::Left(true)),
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
        for folder in params.workspace_folders.unwrap_or_default() {
            self.store
                .find_documents(&folder.uri.to_file_path().unwrap())
        }
        self.send_status()?;

        Ok(())
    }

    fn get_status(&self) -> lsp_ext::ServerStatusParams {
        lsp_ext::ServerStatusParams {
            health: crate::lsp_ext::Health::Ok,
            quiescent: !self.indexing,
            message: None,
        }
    }

    fn send_status(&self) -> anyhow::Result<()> {
        self.client
            .send_notification::<lsp_ext::ServerStatusNotification>(self.get_status())?;
        Ok(())
    }

    fn register_file_watching(&mut self) -> anyhow::Result<()> {
        // TODO: Check if this is enough to delete the watcher
        self.store.watcher = None;

        let tx = self.internal_tx.clone();
        let watcher = notify::recommended_watcher(move |ev: Result<_, _>| {
            if let Ok(ev) = ev {
                let _ = tx.send(InternalMessage::FileEvent(ev));
            }
        });

        if let Ok(mut watcher) = watcher {
            for include_dir_path in self.store.environment.options.includes_directories.iter() {
                if include_dir_path.exists() {
                    watcher.watch(include_dir_path, notify::RecursiveMode::Recursive)?;
                }
            }
            self.store.register_watcher(watcher);
        }

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

    fn did_change_configuration(
        &mut self,
        params: DidChangeConfigurationParams,
    ) -> anyhow::Result<()> {
        if self
            .store
            .environment
            .client_capabilities
            .has_pull_configuration_support()
        {
            self.spawn(move |server| {
                let _ = server.pull_config();
            });
        } else {
            let options = self.fork().parse_options(params.settings)?;
            self.store.environment.options = Arc::new(options);
            self.config_pulled = true;
            self.reparse_all()?;
        }

        Ok(())
    }

    fn reparse_all(&mut self) -> anyhow::Result<()> {
        self.indexing = true;
        self.send_status()?;
        self.parse_directories();
        let main_uri = self.store.environment.options.get_main_path_uri();
        let now = Instant::now();
        if let Some(main_uri) = main_uri {
            let document = self
                .store
                .get(&main_uri)
                .expect("Main Path does not exist.");
            self.store
                .handle_open_document(document.uri, document.text, &mut self.parser)
                .expect("Couldn't parse file");
        } else {
            self.client
                .send_notification::<ShowMessage>(ShowMessageParams {
                    message: "Invalid MainPath setting.\nPlease make sure it is valid.".to_string(),
                    typ: MessageType::WARNING,
                })?;
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
        }
        self.store.find_all_references();
        self.store.first_parse = false;
        eprintln!("Reparsed all the files in {:.2?}", now.elapsed());
        self.indexing = false;
        self.send_status()?;

        Ok(())
    }

    fn parse_directories(&mut self) {
        let directories = self.store.environment.options.includes_directories.clone();
        for path in directories {
            if !path.exists() {
                self.client
                    .send_notification::<ShowMessage>(ShowMessageParams {
                        message: format!(
                            "Invalid IncludeDirectory path: {}",
                            path.to_str().unwrap_or_default()
                        ),
                        typ: MessageType::WARNING,
                    })
                    .unwrap_or_default();
                continue;
            }
            self.store.find_documents(&path);
        }
    }

    /// Check if a [uri](Url) is know or not. If it is not, scan its parent folder and analyze all the documents that
    /// have not been scanned.
    ///
    /// # Arguments
    ///
    /// * `uri` - [Uri](Url) of the document to test for.
    fn read_unscanned_document(&mut self, uri: Arc<Url>) {
        if self.store.documents.get(&uri).is_some() {
            return;
        }
        let path = uri.to_file_path().unwrap();
        let parent_dir = path.parent().unwrap().to_path_buf();
        self.store.find_documents(&parent_dir);
        let uris: Vec<Url> = self
            .store
            .documents
            .keys()
            .map(|uri| uri.as_ref().clone())
            .collect();
        for uri in uris {
            let document = self.store.documents.get(&uri);
            if let Some(document) = document {
                if !document.parsed {
                    self.store
                        .handle_open_document(
                            document.uri.clone(),
                            document.text.clone(),
                            &mut self.parser,
                        )
                        .unwrap();
                }
            }
        }
    }

    fn completion(&mut self, id: RequestId, mut params: CompletionParams) -> anyhow::Result<()> {
        utils::normalize_uri(&mut params.text_document_position.text_document.uri);
        let uri = Arc::new(params.text_document_position.text_document.uri.clone());
        self.read_unscanned_document(uri.clone());

        self.handle_feature_request(id, params, uri, providers::completion::provide_completions)?;
        Ok(())
    }

    fn hover(&mut self, id: RequestId, mut params: HoverParams) -> anyhow::Result<()> {
        utils::normalize_uri(&mut params.text_document_position_params.text_document.uri);
        let uri = Arc::new(
            params
                .text_document_position_params
                .text_document
                .uri
                .clone(),
        );
        self.read_unscanned_document(uri.clone());

        self.handle_feature_request(id, params, uri, providers::hover::provide_hover)?;
        Ok(())
    }

    fn definition(
        &mut self,
        id: RequestId,
        mut params: GotoDefinitionParams,
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

        self.handle_feature_request(id, params, uri, providers::definition::provide_definition)?;
        Ok(())
    }

    fn semantic_tokens(
        &mut self,
        id: RequestId,
        mut params: SemanticTokensParams,
    ) -> anyhow::Result<()> {
        utils::normalize_uri(&mut params.text_document.uri);
        let uri = Arc::new(params.text_document.uri.clone());
        self.read_unscanned_document(uri.clone());

        self.handle_feature_request(
            id,
            params,
            uri,
            providers::semantic_tokens::provide_semantic_tokens,
        )?;
        Ok(())
    }

    fn signature_help(
        &mut self,
        id: RequestId,
        mut params: SignatureHelpParams,
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

        self.handle_feature_request(
            id,
            params,
            uri,
            providers::signature_help::provide_signature_help,
        )?;
        Ok(())
    }

    fn reference(&mut self, id: RequestId, mut params: ReferenceParams) -> anyhow::Result<()> {
        utils::normalize_uri(&mut params.text_document_position.text_document.uri);
        let uri = Arc::new(params.text_document_position.text_document.uri.clone());
        self.read_unscanned_document(uri.clone());

        self.handle_feature_request(id, params, uri, providers::reference::provide_reference)?;
        Ok(())
    }

    fn document_symbol(
        &mut self,
        id: RequestId,
        mut params: DocumentSymbolParams,
    ) -> anyhow::Result<()> {
        utils::normalize_uri(&mut params.text_document.uri);
        let uri = Arc::new(params.text_document.uri.clone());
        self.read_unscanned_document(uri.clone());

        self.handle_feature_request(
            id,
            params,
            uri,
            providers::document_symbol::provide_document_symbol,
        )?;
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

    fn handle_file_event(&mut self, event: notify::Event) {
        match event.kind {
            notify::EventKind::Create(_) => {
                for path in event.paths {
                    let _ = self.store.load(path, &mut self.parser);
                }
            }
            notify::EventKind::Modify(modify_event) => {
                let uri = Url::from_file_path(event.paths[0].clone());
                if uri.is_err() {
                    return;
                }
                let mut uri = uri.unwrap();
                utils::normalize_uri(&mut uri);
                match modify_event {
                    notify::event::ModifyKind::Name(_) => {
                        if event.paths[0].is_dir()
                            && self
                                .store
                                .environment
                                .options
                                .is_parent_of_include_dir(&event.paths[0])
                        {
                            // The path of one of the watched directory has changed. We must unwatch it.
                            if let Some(watcher) = &self.store.watcher {
                                watcher
                                    .lock()
                                    .unwrap()
                                    .unwatch(event.paths[0].as_path())
                                    .unwrap_or_default();
                                return;
                            }
                        }
                        let uri = Url::from_file_path(&event.paths[0]);
                        if uri.is_err() {
                            return;
                        }
                        let mut uri = uri.unwrap();
                        utils::normalize_uri(&mut uri);
                        let mut uris = self.store.get_all_files_in_folder(&uri);
                        if uris.is_empty() {
                            if event.paths[0].is_dir() {
                                // The second notification of a folder rename causes an empty vector.
                                // Iterate over all the files of the folder instead.
                                for entry in WalkDir::new(&event.paths[0])
                                    .follow_links(true)
                                    .into_iter()
                                    .filter_map(|e| e.ok())
                                {
                                    if entry.path().is_file() {
                                        let uri = Url::from_file_path(entry.path());
                                        if let Ok(uri) = uri {
                                            uris.push(uri);
                                        }
                                    }
                                }
                            } else {
                                // Assume the event points to a file which has been deleted for the rename.
                                uris.push(uri);
                            }
                        }
                        for uri in uris.iter() {
                            match self.store.get(uri) {
                                Some(_) => {
                                    self.store.remove(uri);
                                }
                                None => {
                                    let _ = self
                                        .store
                                        .load(uri.to_file_path().unwrap(), &mut self.parser);
                                }
                            }
                        }
                    }
                    _ => {
                        if let Some(document) = self.store.documents.get(&uri) {
                            let _ = self.store.handle_open_document(
                                Arc::new(uri),
                                document.text.clone(),
                                &mut self.parser,
                            );
                        }
                    }
                }
            }
            notify::EventKind::Remove(_) => {
                for mut uri in event.paths.iter().flat_map(Url::from_file_path) {
                    utils::normalize_uri(&mut uri);
                    self.store.remove(&uri);
                }
            }
            notify::EventKind::Any | notify::EventKind::Access(_) | notify::EventKind::Other => {}
        };
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
                                .on::<SemanticTokensFullRequest, _>(|id, params| self.semantic_tokens(id, params))?
                                .on::<SignatureHelpRequest, _>(|id, params| self.signature_help(id, params))?
                                .on::<References, _>(|id, params| self.reference(id, params))?
                                .on::<DocumentSymbolRequest, _>(|id, params| self.document_symbol(id, params))?

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
                                .on::<DidChangeConfiguration, _>(|params| {
                                    self.did_change_configuration(params)
                                })?
                                .default();
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
