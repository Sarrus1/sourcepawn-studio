use lsp_types::CompletionItem;

use self::function_item::FunctionItem;

pub mod function_item;

#[derive(Debug)]
/// Generic representation of an item, which can be converted to a
/// [CompletionItem](lsp_types::CompletionItem), [Location](lsp_types::Location), etc.
pub enum SPItem {
    Function(FunctionItem),
}

// impl fmt::Debug for SPItem {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         f.debug_struct("SPItem")
//             .field("name", &self.name)
//             .field("kind", &self.kind)
//             .field("type", &self.type_)
//             .field("range", &self.range)
//             .field("full range", &self.full_range)
//             .field("description", &self.description)
//             .field("file_path", &self.uri_string)
//             .finish()
//     }
// }

pub fn to_completion(sp_item: &SPItem) -> CompletionItem {
    match sp_item {
        SPItem::Function(function_item) => function_item::to_completion(function_item),
    }
}
