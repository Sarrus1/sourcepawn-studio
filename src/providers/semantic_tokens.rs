use lsp_types::{
    SemanticTokenModifier, SemanticTokenType, SemanticTokens, SemanticTokensLegend,
    SemanticTokensParams,
};

use crate::spitem::{get_all_items, SPItem};

use self::builder::SemanticTokensBuilder;

use super::FeatureRequest;

mod builder;

pub fn provide_semantic_tokens(
    request: FeatureRequest<SemanticTokensParams>,
) -> Option<SemanticTokens> {
    let uri = request.uri;
    let all_items = get_all_items(&request.store, false);
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
        let item_lock = item.read().unwrap();
        match &*item_lock {
            SPItem::Enum(enum_item) => builder.build_enum(enum_item, &uri),
            SPItem::Variable(variable_item) => builder.build_global_variable(variable_item, &uri),
            SPItem::Define(define_item) => builder.build_define(define_item, &uri),
            SPItem::Function(function_item) => builder.build_function(function_item, &uri),
            SPItem::Methodmap(mm_item) => builder.build_methodmap(mm_item, &uri),
            SPItem::EnumStruct(es_item) => builder.build_enum_struct(es_item, &uri),
            _ => Ok(()),
        }
        .unwrap_or_default();
    }

    Some(builder.build(None))
}
