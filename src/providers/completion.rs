use lsp_types::{CompletionList, CompletionParams};

use crate::spitem::get_all_items;

use self::{
    context::is_method_call,
    getters::{get_method_completions, get_non_method_completions},
};

use super::FeatureRequest;

mod context;
mod getters;
mod matchtoken;

pub fn provide_completions(request: FeatureRequest<CompletionParams>) -> Option<CompletionList> {
    let document = request.store.get(&request.uri)?;
    let all_items = get_all_items(&request.store)?;
    let position = request.params.text_document_position.position;
    let line = document.line(position.line)?;

    if !is_method_call(line, position) {
        return get_non_method_completions(all_items, request.params);
    }

    get_method_completions(all_items, line, position, request)
}
