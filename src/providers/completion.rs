use lsp_types::{CompletionItem, CompletionList, CompletionParams};

use crate::spitem::get_all_items;

use super::FeatureRequest;

pub fn provide_completions(request: FeatureRequest<CompletionParams>) -> Option<CompletionList> {
    let all_items = get_all_items(&request.store);

    let all_items = all_items?;
    let mut items: Vec<CompletionItem> = Vec::new();
    for sp_item in all_items.iter() {
        let res = sp_item.lock().unwrap().to_completion(&request.params);
        if let Some(res) = res {
            items.push(res);
        }
    }

    Some(CompletionList {
        items,
        ..Default::default()
    })
}
