use lsp_types::{CompletionItem, CompletionList, CompletionParams, Url};

use crate::spitem::{get_all_items, to_completion};

use super::FeatureRequest;

pub fn provide_completions(request: FeatureRequest<CompletionParams>) -> Option<CompletionList> {
    let main_path = request.store.environment.options.main_path.clone();
    let main_path_uri = Url::from_file_path(main_path).expect("Invalid main path");
    let all_items = get_all_items(&request.store, main_path_uri);
    let mut items: Vec<CompletionItem> = Vec::new();
    for sp_item in all_items.iter() {
        let res = to_completion(sp_item, &request.params);
        if res.is_some() {
            items.push(res.unwrap());
        }
    }

    Some(CompletionList {
        items,
        ..Default::default()
    })
}
