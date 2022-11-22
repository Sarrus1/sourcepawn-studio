use std::cell::RefCell;
use std::env;
use std::error::Error;

use lsp_types::notification::{DidChangeTextDocument, DidOpenTextDocument, Notification};
use lsp_types::request::Completion;
use lsp_types::{CompletionOptions, OneOf, TextDocumentSyncCapability, TextDocumentSyncKind};
use lsp_types::{InitializeParams, ServerCapabilities};

use crate::providers::RequestHandler;
use lsp_server::{Connection, ExtractError, Message, Request, RequestId};
use store::Store;

mod fileitem;
mod parser;
mod providers;
mod spitem;
mod store;
mod utils;

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

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    // Note that  we must have our logging only write out to stderr.
    eprintln!("starting generic LSP server");
    env::set_var("RUST_BACKTRACE", "1");

    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    // also be implemented to use sockets or HTTP.
    let (connection, io_threads) = Connection::stdio();

    // Run the server and wait for the two threads to end (typically by trigger LSP Exit event).
    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        definition_provider: Some(OneOf::Left(true)),
        completion_provider: Some(CompletionOptions {
            ..Default::default()
        }),
        ..Default::default()
    })
    .unwrap();
    let initialization_params = connection.initialize(server_capabilities)?;
    main_loop(connection, initialization_params)?;
    io_threads.join()?;

    // Shut down gracefully.
    eprintln!("shutting down server");
    Ok(())
}

fn main_loop(
    connection: Connection,
    params: serde_json::Value,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    let _params: InitializeParams = serde_json::from_value(params).unwrap();
    // First initialize the server with the lookahead from the server invocation
    let store = RefCell::new(store::Store::new());
    eprintln!("starting main loop");
    for msg in &connection.receiver {
        eprintln!("got msg: {:?}", msg);
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                eprintln!("got request: {:?}", req);
                match req.method.as_str() {
                    <Completion as lsp_types::request::Request>::METHOD => {
                        request_match!(Completion, store, connection, req);
                    }
                    _ => {
                        eprintln!("Unhandled request {}", req.method);
                    }
                }
            }
            Message::Response(resp) => {
                eprintln!("got response: {:?}", resp);
            }
            Message::Notification(not) => {
                match process_notification(not, &connection, &store) {
                    Ok(()) => continue,
                    Err(err) => eprintln!("An error has occured: {}", err),
                };
            }
        }
    }
    Ok(())
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
