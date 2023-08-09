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
    let all_items = &store.get_all_items(false).0;
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
            SPItem::Enum(enum_item) => builder.build_enum(enum_item, &params.text_document.uri),
            SPItem::Variable(variable_item) => {
                builder.build_global_variable(variable_item, &params.text_document.uri)
            }
            SPItem::Define(define_item) => {
                builder.build_define(define_item, &params.text_document.uri)
            }
            SPItem::Function(function_item) => {
                builder.build_function(function_item, &params.text_document.uri)
            }
            SPItem::Methodmap(mm_item) => {
                builder.build_methodmap(mm_item, &params.text_document.uri)
            }
            SPItem::EnumStruct(es_item) => {
                builder.build_enum_struct(es_item, &params.text_document.uri)
            }
            _ => Ok(()),
        }
        .unwrap_or_default();
    }

    Some(builder.build(None))
}
