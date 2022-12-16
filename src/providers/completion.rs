use lsp_types::{CompletionItem, CompletionList, CompletionParams, Position};

use crate::spitem::{get_all_items, get_items_from_position, SPItem};

use self::{
    context::{get_line_words, is_method_call},
    getters::get_children_of_mm_or_es,
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
        let mut items: Vec<CompletionItem> = Vec::new();
        for sp_item in all_items.iter() {
            let res = sp_item
                .lock()
                .unwrap()
                .to_completion(&request.params, false);
            if let Some(res) = res {
                items.push(res);
            }
        }

        return Some(CompletionList {
            items,
            ..Default::default()
        });
    }

    let words = get_line_words(line, position);
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
            let type_ = item.lock().unwrap().type_();
            for item_ in all_items.iter() {
                if item_.lock().unwrap().name() == type_ {
                    let item_lock = item_.lock().unwrap().clone();
                    match item_lock {
                        SPItem::Methodmap(mm_item) => {
                            return Some(CompletionList {
                                // TODO: Handle inherit here
                                // TODO: Handle static methods
                                items: get_children_of_mm_or_es(
                                    &all_items,
                                    mm_item.name,
                                    request.params,
                                ),
                                ..Default::default()
                            });
                        }
                        SPItem::EnumStruct(es_item) => {
                            return Some(CompletionList {
                                items: get_children_of_mm_or_es(
                                    &all_items,
                                    es_item.name,
                                    request.params,
                                ),
                                ..Default::default()
                            });
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    None
}
