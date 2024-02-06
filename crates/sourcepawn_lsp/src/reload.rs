use std::sync::Arc;
use std::{mem, vec};

use paths::AbsPathBuf;
use vfs::VfsPath;

use crate::{config::Config, GlobalState};

impl GlobalState {
    pub(crate) fn update_configuration(&mut self, config: Config) {
        let old_config = mem::replace(&mut self.config, Arc::new(config));
        if self.config.include_directories() != old_config.include_directories()
            || self.config.root_path() != old_config.root_path()
        {
            let mut roots = vec![VfsPath::from(
                AbsPathBuf::try_from(self.config.root_path().clone()).expect("Bad root path"),
            )];
            roots.extend(
                self.config
                    .include_directories()
                    .iter()
                    .flat_map(|it| AbsPathBuf::try_from(it.clone()).map(VfsPath::from)),
            );
            self.source_root_config.fsc.set_roots(roots);
        }
    }
}
