use std::sync::Arc;
use std::{mem, vec};

use itertools::Itertools;
use vfs::VfsPath;

use crate::{config::Config, GlobalState};

impl GlobalState {
    pub(crate) fn update_configuration(&mut self, config: Config) {
        let old_config = mem::replace(&mut self.config, Arc::new(config));
        if self.config.include_directories() != old_config.include_directories()
            || self.config.root_path() != old_config.root_path()
        {
            let mut roots = vec![VfsPath::from(self.config.root_path().clone())];
            roots.extend(
                self.config
                    .include_directories()
                    .into_iter()
                    .map(VfsPath::from),
            );
            self.source_root_config.fsc.set_roots(roots);
            let mut load = self
                .config
                .include_directories()
                .into_iter()
                .map(vfs::loader::Entry::sp_files_recursively)
                .collect_vec();
            let watch = (0..load.len()).collect_vec();
            load.push(vfs::loader::Entry::sp_files_recursively(
                self.config.root_path().clone(),
            ));
            self.vfs_config_version += 1;
            self.loader.handle.set_config(vfs::loader::Config {
                load,
                watch,
                version: self.vfs_config_version,
            });
        }
    }
}
