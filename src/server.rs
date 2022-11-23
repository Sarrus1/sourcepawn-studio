use crate::{
    environment::Environment, options::Options, providers::RequestHandler, store::Store,
    workspace::Workspace,
};
use std::{cell::RefCell, path::PathBuf, sync::Arc};

use anyhow;
use crossbeam_channel::{Receiver, Sender};
use lsp_server::{Connection, ExtractError, Message, Request, RequestId};
use lsp_types::{
    notification::{DidChangeTextDocument, DidOpenTextDocument, Notification, ShowMessage},
    request::{Completion, WorkspaceConfiguration},
    CompletionOptions, ConfigurationItem, ConfigurationParams, InitializeParams, MessageType,
    OneOf, ServerCapabilities, ShowMessageParams, TextDocumentSyncCapability, TextDocumentSyncKind,
};
use threadpool::ThreadPool;

use crate::client::LspClient;

macro_rules! request_match {
    ($req_type:ty, $store:expr, $connection:expr, $req:expr) => {
        match cast::<$req_type>($req) {
            Ok((id, params)) => {
                let resp = <$req_type>::handle(&mut $store.borrow_mut(), id, params);
                // eprintln!("send response: {:?}", resp);
                $connection.sender.send(Message::Response(resp))?;
                continue;
            }
            Err(err @ ExtractError::JsonError { .. }) => panic!("{:?}", err),
            Err(ExtractError::MethodMismatch(req)) => req,
        };
    };
}

#[derive(Debug)]
enum InternalMessage {
    SetOptions(Arc<Options>),
}

#[derive(Clone)]
struct ServerFork {
    connection: Arc<Connection>,
    internal_tx: Sender<InternalMessage>,
    client: LspClient,
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
}

pub struct Server {
    connection: Arc<Connection>,
    client: LspClient,
    store: RefCell<Store>,
    internal_tx: Sender<InternalMessage>,
    internal_rx: Receiver<InternalMessage>,
    pool: ThreadPool,
    workspace: Workspace,
}

impl Server {
    pub fn new(connection: Connection, current_dir: PathBuf) -> Self {
        let client = LspClient::new(connection.sender.clone());
        let workspace = Workspace::new(Environment::new(Arc::new(current_dir)));
        let (internal_tx, internal_rx) = crossbeam_channel::unbounded();
        Self {
            connection: Arc::new(connection),
            client,
            internal_rx,
            internal_tx,
            store: RefCell::new(Store::new()),
            pool: threadpool::Builder::new().build(),
            workspace,
        }
    }

    fn initialize(&mut self) -> anyhow::Result<()> {
        let server_capabilities = serde_json::to_value(&ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
            definition_provider: Some(OneOf::Left(true)),
            completion_provider: Some(CompletionOptions {
                ..Default::default()
            }),
            ..Default::default()
        })
        .unwrap();
        let initialization_params = self.connection.initialize(server_capabilities)?;
        let params: InitializeParams = serde_json::from_value(initialization_params).unwrap();
        self.workspace.environment.client_capabilities = Arc::new(params.capabilities);
        self.workspace.environment.client_info = params.client_info.map(Arc::new);

        self.spawn(move |server| {
            let _ = server.pull_config();
        });
        let base_path = PathBuf::from(params.workspace_folders.unwrap()[0].uri.path());
        self.store.borrow_mut().find_documents(&base_path);
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
        }
    }

    fn process_messages(&mut self) -> anyhow::Result<()> {
        loop {
            crossbeam_channel::select! {
                            recv(&self.connection.receiver) -> msg => {
                        eprintln!("got msg: {:?}", msg);
                        match msg? {
                            Message::Request(req) => {
                                if self.connection.handle_shutdown(&req)? {
                                    return Ok(());
                                }
                                eprintln!("got request: {:?}", req);
                                match req.method.as_str() {
                                    <Completion as lsp_types::request::Request>::METHOD => {
                                        request_match!(Completion, self.store, self.connection, req);
                                    }
                                    _ => {
                                        eprintln!("Unhandled request {}", req.method);
                                    }
                                }
                            }
                            Message::Response(resp) => {
                                self.client.recv_response(resp)?;
                            }
                            Message::Notification(not) => {
                                match not.method.as_str() {
                                    DidOpenTextDocument::METHOD => self.store.borrow_mut().handle_open_document(&self.connection, not)?,
                                    DidChangeTextDocument::METHOD => {
                                        self.store.borrow_mut().handle_change_document(&self.connection, not)?
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    recv(&self.internal_rx) -> msg => {
                        match msg? {
                            InternalMessage::SetOptions(options) => {
                                self.workspace.environment.options = options;
                                self.store.borrow_mut().parse_directories(&self.workspace.environment.options.includes_directories);
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

fn cast<R>(req: Request) -> Result<(RequestId, R::Params), ExtractError<Request>>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}
