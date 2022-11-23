use lsp_types::{CompletionItem, CompletionList, CompletionParams};

use crate::spitem::to_completion;

use super::FeatureRequest;

pub fn provide_completions(request: FeatureRequest<CompletionParams>) -> Option<CompletionList> {
    let mut items: Vec<CompletionItem> = Vec::new();
    for (_, file_item) in request.store.documents.iter() {
        for sp_item in file_item.sp_items.iter() {
            let res = to_completion(sp_item, &request.params);
            if res.is_some() {
                items.push(res.unwrap());
            }
        }
    }

    Some(CompletionList {
        items,
        ..Default::default()
    })
}
