use std::mem;
use std::sync::Arc;

use crate::{config::Config, GlobalState};

impl GlobalState {
    pub(crate) fn update_configuration(&mut self, config: Config) {
        let old_config = mem::replace(&mut self.config, Arc::new(config));
    }
}
