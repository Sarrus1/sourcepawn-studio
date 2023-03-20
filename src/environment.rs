use std::sync::Arc;

use lsp_types::{ClientCapabilities, ClientInfo};
use uuid::Uuid;

use crate::options::Options;

#[derive(Debug, Clone)]
pub struct Environment {
    pub client_capabilities: Arc<ClientCapabilities>,
    pub client_info: Option<Arc<ClientInfo>>,
    pub options: Arc<Options>,
    pub(super) sp_comp_uuid: Uuid,
}

impl Environment {
    #[must_use]
    pub fn new() -> Self {
        Self {
            client_capabilities: Arc::new(ClientCapabilities::default()),
            client_info: None,
            options: Arc::new(Options::default()),
            sp_comp_uuid: Uuid::new_v4(),
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
