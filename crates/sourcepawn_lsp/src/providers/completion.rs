use crate::providers::completion::{
    getters::get_ctor_completions, include::get_include_completions,
};
use lsp_types::{CompletionItem, CompletionList, CompletionParams};
use semantic_analyzer::is_ctor_call;
use sourcepawn_lexer::{SourcepawnLexer, TokenKind};
use store::Store;
use syntax::range_contains_pos;

use self::{
    context::{is_callback_completion_request, is_doc_completion, is_method_call},
    getters::{get_callback_completions, get_method_completions, get_non_method_completions},
    include::is_include_statement,
};

pub(crate) mod context;
mod defaults;
mod getters;
mod include;
mod matchtoken;

pub(crate) fn provide_completions(
    store: &Store,
    params: CompletionParams,
) -> Option<CompletionList> {
    log::debug!("Providing completions.");
    let uri = &params.text_document_position.text_document.uri;
    let document = store.documents.get(uri)?;
    let all_items = store.get_all_items(uri, false);
    let position = &params.text_document_position.position;
    let line = document.line(position.line)?;
    let pre_line: String = line.chars().take(position.character as usize).collect();

    let lexer = SourcepawnLexer::new(&document.text);
    for token in lexer {
        if range_contains_pos(&token.range, position) {
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
                    return get_include_completions(store, include_st);
                }
                return None;
            }
            '.' | ':' => {
                return get_method_completions(store, &params, all_items, &pre_line);
            }
            ' ' => {
                if is_ctor_call(&pre_line) {
                    return get_ctor_completions(all_items, params);
                }
                return None;
            }
            '$' => {
                if is_callback_completion_request(params.context) {
                    return get_callback_completions(
                        all_items,
                        params.text_document_position.position,
                    );
                }
                return None;
            }
            '*' => {
                if let (Some(item), Some(line)) = (
                    is_doc_completion(&pre_line, position, &all_items),
                    document.line(position.line + 1),
                ) {
                    return item.read().doc_completion(line);
                }
                return None;
            }
            _ => {
                // In the last case, the user might be picking on an unfinished completion:
                // If the user starts to type the completion for a method, clicks elsewhere,
                // then starts typing the name of the method again, we will end up in this block.
                // Therefore, this block must cover all possibilities.
                let include_st = is_include_statement(&pre_line);
                if let Some(include_st) = include_st {
                    return get_include_completions(store, include_st);
                }

                if is_callback_completion_request(params.context.clone()) {
                    return get_callback_completions(
                        all_items,
                        params.text_document_position.position,
                    );
                }

                if !is_method_call(&pre_line) {
                    if is_ctor_call(&pre_line) {
                        return get_ctor_completions(all_items, params);
                    }
                    return get_non_method_completions(all_items, params);
                }

                return get_method_completions(store, &params, all_items, &pre_line);
            }
        }
    }

    get_non_method_completions(all_items, params)
}

pub(crate) fn resolve_completion_item(
    store: &Store,
    completion_item: CompletionItem,
) -> Option<CompletionItem> {
    let mut completion_item = completion_item;
    // TODO: Fix with a path interner, that is passed with the key.
    // let key = completion_item.data.clone()?;
    // if let Some(sp_item) = store.get_item_from_key(key.to_string().replace('"', "")) {
    //     let sp_item = &*sp_item.read();
    //     completion_item.detail = Some(sp_item.formatted_text());
    //     completion_item.documentation = sp_item.documentation();
    //     return Some(completion_item);
    // }

    None
}
