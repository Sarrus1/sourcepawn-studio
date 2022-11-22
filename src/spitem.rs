use lsp_types::{CompletionItem, CompletionParams};

use self::function_item::FunctionItem;

pub mod function_item;

#[derive(Debug)]
/// Generic representation of an item, which can be converted to a
/// [CompletionItem](lsp_types::CompletionItem), [Location](lsp_types::Location), etc.
pub enum SPItem {
    Function(FunctionItem),
}

pub fn to_completion(sp_item: &SPItem, params: &CompletionParams) -> Option<CompletionItem> {
    match sp_item {
        SPItem::Function(function_item) => function_item::to_completion(function_item, params),
    }
}
