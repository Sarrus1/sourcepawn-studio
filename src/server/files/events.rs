use lsp_types::Url;
use notify::Watcher;
use walkdir::WalkDir;

use crate::{utils, Server};

impl Server {
    pub(crate) fn handle_file_event(&mut self, event: notify::Event) {
        match event.kind {
            notify::EventKind::Create(_) => {
                for path in event.paths {
                    let _ = self.store.load(path, &mut self.parser);
                }
            }
            notify::EventKind::Modify(modify_event) => {
                let uri = Url::from_file_path(event.paths[0].clone());
                if uri.is_err() {
                    return;
                }
                let mut uri = uri.unwrap();
                utils::normalize_uri(&mut uri);
                match modify_event {
                    notify::event::ModifyKind::Name(_) => {
                        if event.paths[0].is_dir()
                            && self
                                .store
                                .environment
                                .options
                                .is_parent_of_include_dir(&event.paths[0])
                        {
                            // The path of one of the watched directory has changed. We must unwatch it.
                            if let Some(watcher) = &self.store.watcher {
                                watcher
                                    .lock()
                                    .unwrap()
                                    .unwatch(event.paths[0].as_path())
                                    .unwrap_or_default();
                                return;
                            }
                        }
                        let uri = Url::from_file_path(&event.paths[0]);
                        if uri.is_err() {
                            return;
                        }
                        let mut uri = uri.unwrap();
                        utils::normalize_uri(&mut uri);
                        let mut uris = self.store.get_all_files_in_folder(&uri);
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
                            match self.store.get(uri) {
                                Some(_) => {
                                    self.store.remove(uri, &mut self.parser);
                                }
                                None => {
                                    let _ = self
                                        .store
                                        .load(uri.to_file_path().unwrap(), &mut self.parser);
                                }
                            }
                        }
                    }
                    _ => {
                        if self.store.documents.contains_key(&uri) {
                            let _ = self
                                .store
                                .reload(uri.to_file_path().unwrap(), &mut self.parser);
                        }
                    }
                }
            }
            notify::EventKind::Remove(_) => {
                for mut uri in event.paths.iter().flat_map(Url::from_file_path) {
                    utils::normalize_uri(&mut uri);
                    self.store.remove(&uri, &mut self.parser);
                }
            }
            notify::EventKind::Any | notify::EventKind::Access(_) | notify::EventKind::Other => {}
        };
    }
}
