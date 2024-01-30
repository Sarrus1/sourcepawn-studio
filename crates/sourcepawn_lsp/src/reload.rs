use std::sync::Arc;
use std::{mem, vec};

use lsp_types::Url;

use crate::{config::Config, GlobalState};

impl GlobalState {
    pub(crate) fn update_configuration(&mut self, config: Config) {
        let old_config = mem::replace(&mut self.config, Arc::new(config));
        if self.config.include_directories() != old_config.include_directories()
            || self.config.root_path() != old_config.root_path()
        {
            let mut roots =
                vec![Url::from_file_path(self.config.root_path()).expect("invalid root path")];
            roots.extend(
                self.config
                    .include_directories()
                    .iter()
                    .flat_map(Url::from_file_path),
            );
            self.source_root_config.fsc.set_roots(roots);
        }
    }
}
