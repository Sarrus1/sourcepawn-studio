use lsp_types::{CompletionItem, CompletionList, CompletionParams};

use crate::spitem::get_all_items;

use super::FeatureRequest;

pub fn provide_completions(request: FeatureRequest<CompletionParams>) -> Option<CompletionList> {
    let all_items = get_all_items(&request.store);
    let mut items: Vec<CompletionItem> = Vec::new();
    for sp_item in all_items.iter() {
        let res = sp_item.to_completion(&request.params);
        if res.is_some() {
            items.push(res.unwrap());
        }
    }

    Some(CompletionList {
        items,
        ..Default::default()
    })
}
