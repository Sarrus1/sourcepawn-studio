use std::sync::{Arc, RwLock};

use lsp_types::{CompletionItem, CompletionList, CompletionParams, Position, Range};

use crate::{
    providers::FeatureRequest,
    spitem::{get_items_from_position, SPItem},
};

use super::context::get_line_words;

/// Return a [CompletionList](lsp_types::CompletionList) of all non method completions (that don't come
/// after a `.` or `::`).
///
/// # Arguments
///
/// * `all_items` - Vector of [SPItem](crate::spitem::SPItem).
/// * `params` - [Parameters](lsp_types::completion::CompletionParams) of the completion request.
pub(super) fn get_non_method_completions(
    all_items: Vec<Arc<RwLock<SPItem>>>,
    params: CompletionParams,
) -> Option<CompletionList> {
    let mut items: Vec<CompletionItem> = Vec::new();
    for sp_item in all_items.iter() {
        let res = sp_item.read().unwrap().to_completions(&params, false);
        items.extend(res);
    }

    Some(CompletionList {
        items,
        ..Default::default()
    })
}

/// Return a [CompletionList](lsp_types::CompletionList) of all callback completions.
///
/// # Arguments
///
/// * `all_items` - Vector of [SPItem](crate::spitem::SPItem).
/// * `position` - [Position] of the completion request.
pub(super) fn get_callback_completions(
    all_items: Vec<Arc<RwLock<SPItem>>>,
    position: Position,
) -> Option<CompletionList> {
    // This range is used to replace the "$" that has been inserted as a trigger for the completion.
    let range = Range::new(
        Position::new(position.line, position.character - 1),
        Position::new(position.line, position.character + 1),
    );
    let mut items = vec![];
    for item in all_items.iter() {
        match &*item.read().unwrap() {
            SPItem::Typedef(typedef_item) => {
                if let Some(completion) = typedef_item.to_snippet_completion(range) {
                    items.push(completion);
                }
            }
            SPItem::Typeset(typeset_item) => {
                items.extend(typeset_item.to_snippet_completion(range))
            }
            SPItem::Function(function_item) => {
                if let Some(completion) = function_item.to_snippet_completion(range) {
                    items.push(completion);
                }
            }
            _ => {}
        }
    }

    Some(CompletionList {
        items,
        ..Default::default()
    })
}

/// Return a [CompletionList](lsp_types::CompletionList) of all constructor completions.
///
/// # Arguments
///
/// * `all_items` - Vector of [SPItem](crate::spitem::SPItem).
/// * `params` - [Parameters](lsp_types::completion::CompletionParams) of the completion request.
pub(super) fn get_ctr_completions(
    all_items: Vec<Arc<RwLock<SPItem>>>,
    params: CompletionParams,
) -> Option<CompletionList> {
    let mut items = vec![];
    for ctr in all_items
        .iter()
        .filter_map(|item| item.read().unwrap().ctr())
    {
        items.extend(ctr.read().unwrap().to_completions(&params, true))
    }
    Some(CompletionList {
        items,
        ..Default::default()
    })
}

/// Return a [CompletionList](lsp_types::CompletionList) of all method completions (that should come
/// after a `.` or `::`).
///
/// # Arguments
///
/// * `all_items` - Vector of [SPItem](crate::spitem::SPItem).
/// * `pre_line` - Prefix line of the document to analyze.
/// * `position` - [Position](lsp_types::Position) of the request.
/// * `params` - [Parameters](lsp_types::completion::CompletionParams) of the completion request.
pub(super) fn get_method_completions(
    all_items: Vec<Arc<RwLock<SPItem>>>,
    pre_line: &str,
    position: Position,
    request: FeatureRequest<CompletionParams>,
) -> Option<CompletionList> {
    let words = get_line_words(pre_line, position);
    for word in words.into_iter().flatten().rev() {
        let word_pos = Position {
            line: word.range.start.line,
            character: ((word.range.start.character + word.range.end.character) / 2),
        };
        let items = get_items_from_position(
            &request.store,
            word_pos,
            request
                .params
                .text_document_position
                .text_document
                .uri
                .clone(),
        );
        if items.is_empty() {
            continue;
        }
        for item in items.iter() {
            let type_ = item.read().unwrap().type_();
            let type_item = all_items
                .iter()
                .find(|type_item| type_item.read().unwrap().name() == type_);
            if type_item.is_none() {
                continue;
            }
            let type_item = type_item.unwrap();
            match type_item.read().unwrap().clone() {
                SPItem::Methodmap(mm_item) => {
                    let mut children = mm_item.children;
                    extend_children(&mut children, &mm_item.parent);
                    let mut items = vec![];
                    for child in children.iter() {
                        match &*child.read().unwrap() {
                            SPItem::Function(method_item) => {
                                if method_item.is_ctr() {
                                    // We don't want constructors here.
                                    continue;
                                }
                                if is_static_call(item, type_item) {
                                    // We are trying to call static methods.
                                    if method_item.is_static() {
                                        items.extend(
                                            method_item.to_completions(&request.params, true),
                                        );
                                    }
                                } else if !method_item.is_static() {
                                    // We are trying to call non static methods.
                                    items.extend(method_item.to_completions(&request.params, true));
                                }
                            }
                            SPItem::Property(property_item) => {
                                items.extend(property_item.to_completion(&request.params, true))
                            }
                            _ => {}
                        }
                    }
                    return Some(CompletionList {
                        items,
                        ..Default::default()
                    });
                }
                SPItem::EnumStruct(es_item) => {
                    let mut items = vec![];
                    for child in es_item.children.iter() {
                        items.extend(child.read().unwrap().to_completions(&request.params, true));
                    }
                    return Some(CompletionList {
                        items,
                        ..Default::default()
                    });
                }
                _ => {}
            }
        }
    }

    None
}

fn extend_children(children: &mut Vec<Arc<RwLock<SPItem>>>, mm_item: &Option<Arc<RwLock<SPItem>>>) {
    if let Some(mm_item) = mm_item {
        if let SPItem::Methodmap(mm_item) = &*mm_item.read().unwrap() {
            children.extend(mm_item.children.clone());
            extend_children(children, &mm_item.parent);
        }
    }
}

/// Return whether or not the method call is a static call.
///
/// If the name of the method caller is the same as the name of the type, it's a static call.
///
/// # Example
///
/// ```
/// Database.Connect(); // <- Static call
/// cvFoo.GetStringValue(); // <- Non static call
/// ```
///
/// # Arguments
///
/// * `item` - [SPItem](crate::spitem::SPItem) of the call origin.
/// * `type_item` - [SPItem](crate::spitem::SPItem) associated with the type.
fn is_static_call(item: &Arc<RwLock<SPItem>>, type_item: &Arc<RwLock<SPItem>>) -> bool {
    item.read().unwrap().name() == type_item.read().unwrap().name()
}
