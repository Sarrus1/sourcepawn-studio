use std::sync::{Arc, Mutex};

use crate::spitem::SPItem;

pub(super) struct Inherit {
    item: Option<Arc<Mutex<SPItem>>>,
}

impl Iterator for Inherit {
    type Item = Arc<Mutex<SPItem>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.item.is_none() {
            return None;
        }
        let item = self.item.clone().unwrap();
        let item = item.lock().unwrap();
        match &*item {
            SPItem::Methodmap(inherit) => match &inherit.parent {
                Some(parent) => {
                    self.item = Some(parent.clone());
                    return Some(parent.clone());
                }
                None => return None,
            },
            _ => return None,
        }
    }
}

pub(super) fn find_inherit(all_items: &Vec<Arc<Mutex<SPItem>>>, parent: &SPItem) -> Inherit {
    let mut inherit = None;
    for item_ in all_items.iter() {
        let item_lock = item_.lock().unwrap();
        match &*item_lock {
            SPItem::Methodmap(mm_item) => {
                if mm_item.name == parent.type_() {
                    inherit = Some(item_.clone());
                    break;
                }
            }
            _ => {}
        }
    }

    Inherit { item: inherit }
}
