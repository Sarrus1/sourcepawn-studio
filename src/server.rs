use crate::{options::Options, providers::RequestHandler, store::Store};
use std::{
    cell::RefCell,
    sync::{atomic::AtomicI32, Arc},
};

use anyhow;
use lsp_server::{Connection, ExtractError, Message, Request, RequestId};
use lsp_types::{
    notification::{DidChangeTextDocument, DidOpenTextDocument, Notification, ShowMessage},
    request::{Completion, WorkspaceConfiguration},
    CompletionOptions, ConfigurationItem, ConfigurationParams, InitializeParams, MessageType,
    OneOf, ServerCapabilities, ShowMessageParams, TextDocumentSyncCapability, TextDocumentSyncKind,
};
use serde::{de::DeserializeOwned, Serialize};
use std::error::Error;
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

#[derive(Clone)]
struct ServerFork {
    connection: Arc<Connection>,
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
                Some(self.parse_options(value)?);
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
    initalize_params: Option<InitializeParams>,
    store: RefCell<Store>,
    options: Option<Options>,
    next_id: AtomicI32,
    pool: ThreadPool,
}

impl Server {
    pub fn new(connection: Connection) -> Self {
        let client = LspClient::new(connection.sender.clone());
        Self {
            connection: Arc::new(connection),
            client,
            initalize_params: None,
            store: RefCell::new(Store::new()),
            options: None,
            next_id: AtomicI32::new(1),
            pool: threadpool::Builder::new().build(),
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
        eprintln!("Init params {:?}", initialization_params.to_string());
        self.initalize_params = serde_json::from_value(initialization_params).unwrap();
        self.spawn(move |server| {
            let _ = server.pull_config();
        });
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
                            // Assume we only receive Options here.
                            self.client.recv_response(resp)?;
                            eprintln!("got response");
                        }
                        Message::Notification(not) => {
                            match process_notification(not, &self.connection, &self.store) {
                                Ok(()) => continue,
                                Err(err) => eprintln!("An error has occured: {}", err),
                            };
                        }
                    }
                }
            }
        }
    }

    pub fn send_request<R>(&self, params: R::Params) -> anyhow::Result<()>
    where
        R: lsp_types::request::Request,
        R::Params: Serialize,
        R::Result: DeserializeOwned,
    {
        let id = RequestId::from(
            self.next_id
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst),
        );

        self.connection
            .sender
            .send(Request::new(id, R::METHOD.to_string(), params).into())?;
        Ok(())
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        self.initialize()?;
        self.process_messages()?;
        Ok(())
    }
}

fn process_notification(
    not: lsp_server::Notification,
    connection: &Connection,
    store: &RefCell<Store>,
) -> Result<(), Box<dyn Error>> {
    eprintln!("got notification: {:?}", not);
    match not.method.as_str() {
        DidOpenTextDocument::METHOD => store.borrow_mut().handle_open_document(connection, not)?,
        DidChangeTextDocument::METHOD => {
            store.borrow_mut().handle_change_document(connection, not)?
        }
        _ => {}
    }
    Ok(())
}

fn cast<R>(req: Request) -> Result<(RequestId, R::Params), ExtractError<Request>>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}
