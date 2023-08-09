use notify::Watcher;

use crate::{server::InternalMessage, Server};

impl Server {
    pub(crate) fn register_file_watching(&mut self) -> anyhow::Result<()> {
        // TODO: Check if this is enough to delete the watcher
        self.store.write().watcher = None;

        let tx = self.internal_tx.clone();
        let watcher = notify::recommended_watcher(move |ev: Result<_, _>| {
            if let Ok(ev) = ev {
                let _ = tx.send(InternalMessage::FileEvent(ev));
            }
        });

        if let Ok(mut watcher) = watcher {
            for include_dir_path in self
                .store
                .read()
                .environment
                .options
                .includes_directories
                .iter()
            {
                if include_dir_path.exists() {
                    watcher.watch(include_dir_path, notify::RecursiveMode::Recursive)?;
                }
            }
            self.store.write().register_watcher(watcher);
        }

        Ok(())
    }
}
