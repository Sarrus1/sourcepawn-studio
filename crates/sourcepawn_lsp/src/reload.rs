use std::sync::Arc;
use std::{mem, vec};

use flycheck::{FlycheckConfig, FlycheckHandle};
use fxhash::FxHashMap;
use itertools::Itertools;
use paths::AbsPathBuf;
use vfs::VfsPath;

use crate::{config::Config, GlobalState};

use stdx::format_to;

use crate::lsp_ext;

impl GlobalState {
    pub(crate) fn is_quiescent(&self) -> bool {
        !(self.last_reported_status.is_none()
            || self.vfs_progress_config_version < self.vfs_config_version
            || self.vfs_progress_n_done < self.vfs_progress_n_total)
    }

    pub(crate) fn current_status(&self) -> lsp_ext::ServerStatusParams {
        let mut status = lsp_ext::ServerStatusParams {
            health: lsp_ext::Health::Ok,
            quiescent: self.is_quiescent(),
            message: None,
        };
        let mut message = String::new();

        if let Some(err) = &self.config_errors {
            status.health = lsp_ext::Health::Warning;
            format_to!(message, "{err}\n");
        }

        status
    }

    pub(crate) fn update_configuration(&mut self, config: Config, initialization: bool) {
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
        if !initialization
            && (self.config.compiler_path() != old_config.compiler_path()
                || self.config.compiler_arguments() != old_config.compiler_arguments()
                || self.config.include_directories() != old_config.include_directories())
        {
            self.reload_flycheck();
        }
    }

    pub fn reload_flycheck(&mut self) {
        let analysis = self.analysis_host.analysis();
        let Some(compiler_path) = self.config.compiler_path() else {
            return;
        };
        let Ok(graph) = analysis.graph() else {
            // FIXME: report error
            return;
        };
        let tempdir_path = AbsPathBuf::try_from(self.flycheck_tempdir.path().to_path_buf())
            .expect("Failed to convert tempdir path to AbsPathBuf.");
        let mut flycheck = FxHashMap::default();
        graph.subgraphs_with_roots().keys().for_each(|root| {
            let root = *root;
            let sender = self.flycheck_sender.clone();
            flycheck.insert(
                root,
                FlycheckHandle::spawn(
                    root.0,
                    Box::new(move |msg| sender.send(msg).unwrap()),
                    FlycheckConfig::new(
                        compiler_path.to_owned(),
                        self.config.compiler_arguments(),
                        self.config.include_directories().clone(),
                    ),
                    self.vfs
                        .read()
                        .file_path(root)
                        .as_path()
                        .unwrap()
                        .to_owned(),
                    tempdir_path.clone(),
                ),
            );
        });

        self.flycheck = Arc::new(flycheck);
    }
}
