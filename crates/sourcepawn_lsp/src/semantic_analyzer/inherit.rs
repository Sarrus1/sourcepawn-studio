use parking_lot::RwLock;
use std::sync::Arc;

use crate::spitem::SPItem;

pub(super) struct Inherit {
    item: Option<Arc<RwLock<SPItem>>>,
}

impl Iterator for Inherit {
    type Item = Arc<RwLock<SPItem>>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.item.clone()?;
        let item = item.read();
        match &*item {
            SPItem::Methodmap(inherit) => match &inherit.parent {
                Some(parent) => {
                    self.item = Some(parent.clone());
                    Some(parent.clone())
                }
                None => None,
            },
            _ => None,
        }
    }
}

pub(super) fn find_inherit(all_items: &[Arc<RwLock<SPItem>>], parent: &SPItem) -> Inherit {
    let mut inherit = None;
    for item_ in all_items.iter() {
        let item_lock = item_.read();
        if let SPItem::Methodmap(mm_item) = &*item_lock {
            if mm_item.name == parent.type_() {
                inherit = Some(item_.clone());
                break;
            }
        }
    }

    Inherit { item: inherit }
}
