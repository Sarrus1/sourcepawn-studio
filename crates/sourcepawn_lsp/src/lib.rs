mod capabilities;
mod client;
mod diagnostics;
mod dispatch;
pub mod fixture;
mod global_state;
mod handlers {
    pub(crate) mod notification;
    pub(crate) mod request;
}
mod line_index;
mod main_loop;
mod mem_docs;
mod op_queue;
mod progress;
mod reload;
mod task_pool;
mod version;

mod config;
pub mod lsp;

use serde::de::DeserializeOwned;

pub use self::{client::LspClient, global_state::GlobalState};

pub fn from_json<T: DeserializeOwned>(
    what: &'static str,
    json: &serde_json::Value,
) -> anyhow::Result<T> {
    serde_json::from_value(json.clone())
        .map_err(|e| anyhow::format_err!("Failed to deserialize {what}: {e}; {json}"))
}
