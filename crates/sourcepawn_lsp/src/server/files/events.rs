use lsp_types::Url;
use notify::Watcher;
use store::normalize_uri;
use walkdir::WalkDir;

use crate::Server;

impl Server {
    pub(crate) fn handle_file_event(&mut self, event: notify::Event) {
        match event.kind {
            notify::EventKind::Create(_) => {
                for path in event.paths {
                    let Ok(uri) = Url::from_file_path(path.clone()) else {
                        continue;
                    };
                    let _ = self.store.write().load(path);
                    self.reload_diagnostics(uri);
                }
            }
            notify::EventKind::Modify(modify_event) => {
                let Ok(mut uri) = Url::from_file_path(event.paths[0].clone()) else {
                    return;
                };
                normalize_uri(&mut uri);
                match modify_event {
                    notify::event::ModifyKind::Name(_) => {
                        if event.paths[0].is_dir()
                            && self
                                .store
                                .read()
                                .environment
                                .options
                                .is_parent_of_include_dir(&event.paths[0])
                        {
                            // The path of one of the watched directory has changed. We must unwatch it.
                            if let Some(watcher) = &self.store.read().watcher {
                                watcher
                                    .lock()
                                    .unwrap()
                                    .unwatch(event.paths[0].as_path())
                                    .unwrap_or_default();
                                return;
                            }
                        }
                        let Ok(mut uri) = Url::from_file_path(&event.paths[0]) else {
                            return;
                        };
                        normalize_uri(&mut uri);
                        let mut uris = self.store.write().get_all_files_in_folder(&uri);
                        if uris.is_empty() {
                            if event.paths[0].is_dir() {
                                // The second notification of a folder rename causes an empty vector.
                                // Iterate over all the files of the folder instead.
                                for entry in WalkDir::new(&event.paths[0])
                                    .follow_links(true)
                                    .into_iter()
                                    .filter_map(|e| e.ok())
                                {
                                    if entry.path().is_file() {
                                        let uri = Url::from_file_path(entry.path());
                                        if let Ok(uri) = uri {
                                            uris.push(uri);
                                        }
                                    }
                                }
                            } else {
                                // Assume the event points to a file which has been deleted for the rename.
                                uris.push(uri);
                            }
                        }
                        for uri in uris.iter() {
                            if self.store.read().contains_uri(uri) {
                                self.store.write().remove(uri);
                            } else {
                                let _ = self.store.write().load(uri.to_file_path().unwrap());
                            }
                        }
                    }
                    _ => {
                        if self.store.read().contains_uri(&uri) {
                            let _ = self.store.write().reload(uri.to_file_path().unwrap());
                        }
                    }
                }
                self.reload_diagnostics(uri);
            }
            notify::EventKind::Remove(_) => {
                for mut uri in event.paths.iter().flat_map(Url::from_file_path) {
                    normalize_uri(&mut uri);
                    self.store.write().remove(&uri);
                    self.reload_diagnostics(uri);
                }
            }
            notify::EventKind::Any | notify::EventKind::Access(_) | notify::EventKind::Other => {}
        };
    }
}
