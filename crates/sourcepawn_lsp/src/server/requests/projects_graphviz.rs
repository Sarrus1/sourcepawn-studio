use anyhow::bail;
use lsp_server::RequestId;

use crate::{lsp_ext::ProjectsGraphvizParams, Server};

impl Server {
    pub(super) fn projects_graphviz(
        &mut self,
        id: RequestId,
        _params: ProjectsGraphvizParams,
    ) -> anyhow::Result<()> {
        if let Some(graphviz) = self.store.read().represent_graphs() {
            self.run_query(id, move |_store| graphviz);
            return Ok(());
        }

        bail!("Failed to load projects graph.");
    }
}
