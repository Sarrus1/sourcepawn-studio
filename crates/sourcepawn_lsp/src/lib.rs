mod capabilities;
mod client;
mod dispatch;
pub mod fixture;
mod line_index;
mod lsp_ext;
// mod providers;
mod diagnostics;
mod mem_docs;
mod op_queue;
mod reload;
mod server;
mod task_pool;
mod utils;
mod version;

mod config;
pub mod lsp;

use serde::de::DeserializeOwned;
use server::PrimeCachesProgress;
use vfs::FileId;

pub use self::{client::LspClient, server::GlobalState};

pub fn from_json<T: DeserializeOwned>(
    what: &'static str,
    json: &serde_json::Value,
) -> anyhow::Result<T> {
    serde_json::from_value(json.clone())
        .map_err(|e| anyhow::format_err!("Failed to deserialize {what}: {e}; {json}"))
}

#[derive(Debug)]
pub(crate) enum Task {
    Response(lsp_server::Response),
    Retry(lsp_server::Request),
    Diagnostics(Vec<(FileId, Vec<lsp_types::Diagnostic>)>),
    PrimeCaches(PrimeCachesProgress),
}
