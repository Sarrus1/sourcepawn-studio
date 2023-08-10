use fxhash::FxHashSet;
use lsp_types::{Position, Url};
use parking_lot::RwLock;
use std::sync::Arc;
use syntax::{range_contains_pos, SPItem};

use crate::{document::Document, Store};

impl Store {
    pub fn get_all_items(&self, flat: bool) -> Vec<Arc<RwLock<SPItem>>> {
        log::debug!("Getting all items from store. flat: {}", flat);
        let mut all_items = vec![];
        if let Ok(Some(main_path_uri)) = self.environment.options.get_main_path_uri() {
            let mut includes = FxHashSet::default();
            includes.insert(main_path_uri.clone());
            if let Some(document) = self.documents.get(&main_path_uri) {
                self.get_included_files(document, &mut includes);
                for include in includes.iter() {
                    if let Some(document) = self.documents.get(include) {
                        if flat {
                            all_items.extend(document.get_sp_items_flat());
                        } else {
                            all_items.extend(document.get_sp_items())
                        }
                    }
                }
            }
            log::trace!("Done getting {} item(s)", all_items.len());
            return all_items;
        }
        for document in self.documents.values() {
            for item in document.sp_items.iter() {
                all_items.push(item.clone());
            }
        }

        log::trace!(
            "Done getting {} item(s) without the main path.",
            all_items.len()
        );
        all_items
    }

    pub(crate) fn get_included_files(&self, document: &Document, includes: &mut FxHashSet<Url>) {
        for include_uri in document.includes.keys() {
            if includes.contains(include_uri) {
                continue;
            }
            includes.insert(include_uri.clone());
            if let Some(include_document) = self.documents.get(include_uri) {
                self.get_included_files(include_document, includes);
            }
        }
    }

    pub fn get_items_from_position(
        &self,
        position: Position,
        uri: &Url,
    ) -> Vec<Arc<RwLock<SPItem>>> {
        log::debug!(
            "Getting all items from position {:#?} in file {:#?}.",
            position,
            uri
        );
        let uri = Arc::new(uri);
        let all_items = self.get_all_items(true);
        let mut res = vec![];
        for item in all_items.iter() {
            let item_lock = item.read();
            if range_contains_pos(&item_lock.v_range(), &position)
                && item_lock.uri().as_ref().eq(&uri)
            {
                res.push(item.clone());
                continue;
            }
            match item_lock.references() {
                Some(references) => {
                    for reference in references.iter() {
                        if range_contains_pos(&reference.v_range, &position)
                            && (*reference.uri).eq(*uri)
                        {
                            res.push(item.clone());
                            break;
                        }
                    }
                }
                None => {
                    continue;
                }
            }
        }
        log::trace!("Got {} item(s) from position", res.len());

        res
    }

    pub fn get_item_from_key(&self, key: String) -> Option<Arc<RwLock<SPItem>>> {
        log::debug!("Getting item from key {:?}.", key);
        let all_items = self.get_all_items(false);
        let sub_keys: Vec<&str> = key.split('-').collect();
        if sub_keys.is_empty() {
            return None;
        }
        let mut current_item: Option<Arc<RwLock<SPItem>>> = None;
        for key in sub_keys {
            current_item = match current_item {
                Some(item) => item.read().children().and_then(|children| {
                    children
                        .iter()
                        .find(|child| child.read().name() == key)
                        .cloned()
                }),
                None => all_items
                    .iter()
                    .find(|item| item.read().name() == key)
                    .cloned(),
            };

            if current_item.is_none() {
                log::trace!("Did not find a match from key.");
                return None;
            }
        }
        log::debug!("Got {:#?} from key.", current_item);

        current_item
    }
}
