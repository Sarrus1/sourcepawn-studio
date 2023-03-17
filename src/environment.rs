use std::sync::Arc;

use lsp_types::{ClientCapabilities, ClientInfo};

use crate::options::Options;

#[derive(Debug, Clone)]
pub struct Environment {
    pub client_capabilities: Arc<ClientCapabilities>,
    pub client_info: Option<Arc<ClientInfo>>,
    pub options: Arc<Options>,
}

impl Environment {
    #[must_use]
    pub fn new() -> Self {
        Self {
            client_capabilities: Arc::new(ClientCapabilities::default()),
            client_info: None,
            options: Arc::new(Options::default()),
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
