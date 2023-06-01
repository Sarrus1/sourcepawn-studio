use lsp_types::{CompletionList, CompletionParams};

use crate::providers::completion::{
    context::is_ctor_call, getters::get_ctor_completions, include::get_include_completions,
};

use self::{
    context::{is_callback_completion_request, is_doc_completion, is_method_call},
    getters::{get_callback_completions, get_method_completions, get_non_method_completions},
    include::is_include_statement,
};

use super::FeatureRequest;

pub(crate) mod context;
mod defaults;
mod getters;
mod include;
mod matchtoken;

pub fn provide_completions(request: FeatureRequest<CompletionParams>) -> Option<CompletionList> {
    log::debug!("Providing completions with request: {:#?}", request.params);
    let document = request.store.get(&request.uri)?;
    let all_items = request.store.get_all_items(false);
    let position = request.params.text_document_position.position;
    let line = document.line(position.line)?;
    let pre_line: String = line.chars().take(position.character as usize).collect();

    let lexer_literal = document.get_lexer_state(position);
    if lexer_literal.is_some() {
        // TODO: Return only define statements here if in a preprocessor statement.
        return None;
    }

    if let Some(trigger_char) = line.chars().last() {
        // The trigger character allows us to fine control which completion to trigger.
        match trigger_char {
            '<' | '"' | '\'' | '/' | '\\' => {
                let include_st = is_include_statement(&pre_line);
                if let Some(include_st) = include_st {
                    return get_include_completions(request, include_st);
                }
                return None;
            }
            '.' | ':' => {
                return get_method_completions(all_items, &pre_line, position, request);
            }
            ' ' => {
                if is_ctor_call(&pre_line) {
                    return get_ctor_completions(all_items, request.params);
                }
                return None;
            }
            '$' => {
                if is_callback_completion_request(request.params.context) {
                    return get_callback_completions(all_items, position);
                }
                return None;
            }
            '*' => {
                if let Some(item) = is_doc_completion(&pre_line, &position, &all_items) {
                    return item
                        .read()
                        .unwrap()
                        .doc_completion(document.line(position.line + 1).unwrap());
                }
            }
            _ => {
                // In the last case, the user might be picking on an unfinished completion:
                // If the user starts to type the completion for a method, clicks elsewhere,
                // then starts typing the name of the method again, we will end up in this block.
                // Therefore, this block must cover all possibilities.
                let include_st = is_include_statement(&pre_line);
                if let Some(include_st) = include_st {
                    return get_include_completions(request, include_st);
                }

                if is_callback_completion_request(request.params.context.clone()) {
                    return get_callback_completions(all_items, position);
                }

                if !is_method_call(&pre_line) {
                    if is_ctor_call(&pre_line) {
                        return get_ctor_completions(all_items, request.params);
                    }
                    return get_non_method_completions(all_items, request.params);
                }

                return get_method_completions(all_items, &pre_line, position, request);
            }
        }
    }

    None
}
