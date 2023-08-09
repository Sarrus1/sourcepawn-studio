use std::sync::Arc;

use lsp_types::Url;
use uuid::Uuid;

use crate::options::Options;

#[derive(Debug, Clone)]
pub struct Environment {
    pub options: Arc<Options>,
    pub(super) sp_comp_uuid: Uuid,
    pub amxxpawn_mode: bool,
    pub root_uri: Option<Url>,
}

impl Environment {
    #[must_use]
    pub fn new(amxxpawn_mode: bool) -> Self {
        Self {
            options: Arc::new(Options::default()),
            sp_comp_uuid: Uuid::new_v4(),
            amxxpawn_mode,
            root_uri: None,
        }
    }

    pub fn configuration_section(&self) -> &str {
        if self.amxxpawn_mode {
            "AMXXPawnLanguageServer"
        } else {
            "SourcePawnLanguageServer"
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new(false)
    }
}
