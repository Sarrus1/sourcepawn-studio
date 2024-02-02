use crate::{lsp_ext, GlobalState};

impl GlobalState {
    pub(crate) fn is_quiescent(&self) -> bool {
        !self.last_reported_status.is_none()
        // || self.vfs_progress_config_version < self.vfs_config_version
        // || self.vfs_progress_n_done < self.vfs_progress_n_total)
    }

    pub(crate) fn current_status(&self) -> lsp_ext::ServerStatusParams {
        let status = lsp_ext::ServerStatusParams {
            health: lsp_ext::Health::Ok,
            quiescent: self.is_quiescent(),
            message: None,
        };
        // TODO: Fill the status here.
        status
    }
}
