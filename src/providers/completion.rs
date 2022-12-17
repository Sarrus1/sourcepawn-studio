use lsp_types::{CompletionList, CompletionParams};

use crate::{providers::completion::include::get_include_completions, spitem::get_all_items};

use self::{
    context::is_method_call,
    getters::{get_method_completions, get_non_method_completions},
    include::is_include_statement,
};

use super::FeatureRequest;

mod context;
mod getters;
mod include;
mod matchtoken;

pub fn provide_completions(request: FeatureRequest<CompletionParams>) -> Option<CompletionList> {
    let document = request.store.get(&request.uri)?;
    let all_items = get_all_items(&request.store)?;
    let position = request.params.text_document_position.position;
    let line = document.line(position.line)?;
    let sub_line: String = line.chars().take(position.character as usize).collect();

    let include_st = is_include_statement(&sub_line);
    if let Some(include_st) = include_st {
        return get_include_completions(request, include_st);
    }

    if !is_method_call(&sub_line) {
        return get_non_method_completions(all_items, request.params);
    }

    get_method_completions(all_items, &sub_line, position, request)
}
