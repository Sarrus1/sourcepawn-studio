use anyhow::bail;
use lsp_server::RequestId;
use store::normalize_uri;

use crate::{lsp_ext::ProjectMainPathParams, Server};

impl Server {
    pub(super) fn project_main_path(
        &mut self,
        id: RequestId,
        params: ProjectMainPathParams,
    ) -> anyhow::Result<()> {
        let Some(mut uri) = params.uri else {
            bail!("No uri passed to command");
        };
        normalize_uri(&mut uri);
        let Some(file_id) = self.store.read().vfs.get(&uri) else {
            bail!("No file ID found for URI {:?}", uri);
        };
        let Some(root_node) = self.store.read().projects.find_root_from_id(file_id) else {
            bail!("No project root found for file ID {:?}", file_id);
        };
        let main_uri = self.store.read().vfs.lookup(root_node.file_id).clone();
        self.run_query(id, move |_store| main_uri);

        Ok(())
    }
}
