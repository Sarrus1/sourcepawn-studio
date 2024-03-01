use stdx::format_to;

use crate::{lsp_ext, GlobalState};

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
}
