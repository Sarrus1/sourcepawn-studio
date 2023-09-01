use self::builder::SemanticTokensBuilder;
use lsp_types::{
    SemanticTokenModifier, SemanticTokenType, SemanticTokens, SemanticTokensLegend,
    SemanticTokensParams,
};
use store::Store;
use syntax::SPItem;

mod builder;

pub fn provide_semantic_tokens(
    store: &Store,
    params: SemanticTokensParams,
) -> Option<SemanticTokens> {
    let file_id = store.path_interner.get(&params.text_document.uri)?;
    let all_items = &store.get_all_items(&file_id, false);
    if all_items.is_empty() {
        return None;
    }

    let mut builder = SemanticTokensBuilder::new(Some(SemanticTokensLegend {
        token_types: vec![
            SemanticTokenType::VARIABLE,
            SemanticTokenType::ENUM_MEMBER,
            SemanticTokenType::FUNCTION,
            SemanticTokenType::CLASS,
            SemanticTokenType::METHOD,
            SemanticTokenType::MACRO,
            SemanticTokenType::PROPERTY,
            SemanticTokenType::STRUCT,
            SemanticTokenType::ENUM,
        ],
        token_modifiers: vec![
            SemanticTokenModifier::READONLY,
            SemanticTokenModifier::DECLARATION,
            SemanticTokenModifier::DEPRECATED,
            SemanticTokenModifier::MODIFICATION,
        ],
    }));

    for item in all_items.iter() {
        let item_lock = item.read();
        match &*item_lock {
            SPItem::Enum(enum_item) => builder.build_enum(enum_item, file_id),
            SPItem::Variable(variable_item) => {
                builder.build_global_variable(variable_item, file_id)
            }
            SPItem::Define(define_item) => builder.build_define(define_item, file_id),
            SPItem::Function(function_item) => builder.build_function(function_item, file_id),
            SPItem::Methodmap(mm_item) => builder.build_methodmap(mm_item, file_id),
            SPItem::EnumStruct(es_item) => builder.build_enum_struct(es_item, file_id),
            _ => Ok(()),
        }
        .unwrap_or_default();
    }

    Some(builder.build(None))
}
