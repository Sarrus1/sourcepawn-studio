use std::sync::{Arc, Mutex};

use lsp_types::{CompletionItem, CompletionParams};

use crate::spitem::SPItem;

/// Search in a vector of items for the childs of a type and return the associated
/// vector of [CompletionItem](lsp_types::CompletionItem).
///
/// # Arguments
///
/// * `all_item` - Vector of [SPItem](crate::spitem::SPItem).
/// * `parent_name` - Name of the parent.
/// * `params` - [Parameters](lsp_types::completion::CompletionParams) of the completion request.
pub(super) fn get_children_of_mm_or_es(
    all_item: &[Arc<Mutex<SPItem>>],
    parent_name: String,
    params: CompletionParams,
) -> Vec<CompletionItem> {
    let mut res: Vec<CompletionItem> = vec![];
    for item in all_item.iter() {
        let item_lock = item.lock().unwrap();
        if let Some(parent_) = item_lock.parent() {
            if parent_name != parent_.lock().unwrap().name() {
                continue;
            }
            if let Some(completion) = item_lock.to_completion(&params, true) {
                res.push(completion);
            }
        }
    }

    res
}
