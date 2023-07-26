use lsp_types::{CompletionItem, CompletionList, CompletionParams};
use sourcepawn_lexer::{SourcepawnLexer, TokenKind};

use crate::{
    providers::completion::{
        context::is_ctor_call, getters::get_ctor_completions, include::get_include_completions,
    },
    utils,
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

pub(crate) fn provide_completions(
    request: FeatureRequest<CompletionParams>,
) -> Option<CompletionList> {
    log::debug!("Providing completions with request: {:#?}", request.params);
    let document = request.store.documents.get(&request.uri)?;
    let all_items = request.store.get_all_items(false);
    let position = request.params.text_document_position.position;
    let line = document.line(position.line)?;
    let pre_line: String = line.chars().take(position.character as usize).collect();

    let lexer = SourcepawnLexer::new(&document.text);
    for token in lexer {
        if utils::range_contains_pos(token.range, position) {
            match token.token_kind {
                TokenKind::Literal(_) | TokenKind::Comment(_) => return None,
                _ => (),
            }
        }
        if token.range.start.line > position.line {
            break;
        }
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
                return get_method_completions(all_items.0, &pre_line, position, request);
            }
            ' ' => {
                if is_ctor_call(&pre_line) {
                    return get_ctor_completions(all_items.0, request.params);
                }
                return None;
            }
            '$' => {
                if is_callback_completion_request(request.params.context) {
                    return get_callback_completions(all_items.0, position);
                }
                return None;
            }
            '*' => {
                if let Some(item) = is_doc_completion(&pre_line, &position, &all_items.0) {
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
                    return get_callback_completions(all_items.0, position);
                }

                if !is_method_call(&pre_line) {
                    if is_ctor_call(&pre_line) {
                        return get_ctor_completions(all_items.0, request.params);
                    }
                    return get_non_method_completions(all_items.0, request.params);
                }

                return get_method_completions(all_items.0, &pre_line, position, request);
            }
        }
    }

    get_non_method_completions(all_items.0, request.params)
}

pub(crate) fn resolve_completion_item(
    request: FeatureRequest<CompletionItem>,
) -> Option<CompletionItem> {
    let mut completion_item = request.params.clone();

    if let Some(sp_item) = request
        .store
        .get_item_from_key(request.params.data?.to_string().replace('"', ""))
    {
        let sp_item = &*sp_item.read().unwrap();
        completion_item.detail = Some(sp_item.formatted_text());
        completion_item.documentation = sp_item.documentation();
        return Some(completion_item);
    }

    None
}
