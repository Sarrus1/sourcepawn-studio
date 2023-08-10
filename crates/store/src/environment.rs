use lsp_types::Url;
use std::sync::Arc;

use crate::options::Options;

#[derive(Debug, Clone)]
pub struct Environment {
    pub options: Arc<Options>,
    pub amxxpawn_mode: bool,
    pub root_uri: Option<Url>,
}

impl Environment {
    #[must_use]
    pub fn new(amxxpawn_mode: bool) -> Self {
        Self {
            options: Arc::new(Options::default()),
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
