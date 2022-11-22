use lsp_server::{RequestId, Response};
use lsp_types::request::{Completion, Request};

use crate::store::Store;

pub trait RequestHandler: Request {
    fn handle(store: &mut Store, id: RequestId, params: Self::Params) -> Response;
}

impl RequestHandler for Completion {
    fn handle(store: &mut Store, id: RequestId, params: Self::Params) -> Response {
        eprintln!("got completion request #{}: {:?}", id, params);
        let result = store.provide_completions(&params);
        let result = serde_json::to_value(&result).unwrap();
        Response {
            id,
            result: Some(result),
            error: None,
        }
    }
}
