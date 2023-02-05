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
    let all_items = get_all_items(&request.store);
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
            SPItem::Variable(variable_item) => {
                if variable_item.uri.eq(&uri) {
                    builder
                        .push(
                            variable_item.range,
                            SemanticTokenType::VARIABLE,
                            Some(vec![SemanticTokenModifier::DECLARATION]),
                        )
                        .unwrap();
                }
                for ref_ in variable_item.references.iter() {
                    if ref_.uri.eq(&uri) {
                        builder
                            .push(
                                ref_.range,
                                SemanticTokenType::VARIABLE,
                                Some(vec![SemanticTokenModifier::MODIFICATION]),
                            )
                            .unwrap();
                    }
                }
            }
            SPItem::Define(define_item) => {
                if define_item.uri.eq(&uri) {
                    builder
                        .push(
                            define_item.range,
                            SemanticTokenType::MACRO,
                            Some(vec![
                                SemanticTokenModifier::READONLY,
                                SemanticTokenModifier::DECLARATION,
                            ]),
                        )
                        .unwrap();
                }
                for ref_ in define_item.references.iter() {
                    if ref_.uri.eq(&uri) {
                        builder
                            .push(
                                ref_.range,
                                SemanticTokenType::MACRO,
                                Some(vec![SemanticTokenModifier::READONLY]),
                            )
                            .unwrap();
                    }
                }
            }
            SPItem::EnumMember(enum_member_item) => {
                if enum_member_item.uri.eq(&uri) {
                    builder
                        .push(
                            enum_member_item.range,
                            SemanticTokenType::ENUM_MEMBER,
                            Some(vec![
                                SemanticTokenModifier::READONLY,
                                SemanticTokenModifier::DECLARATION,
                            ]),
                        )
                        .unwrap();
                }
                for ref_ in enum_member_item.references.iter() {
                    if ref_.uri.eq(&uri) {
                        builder
                            .push(
                                ref_.range,
                                SemanticTokenType::ENUM_MEMBER,
                                Some(vec![SemanticTokenModifier::READONLY]),
                            )
                            .unwrap();
                    }
                }
            }
            SPItem::Function(function_item) => {
                let type_ = {
                    if function_item.parent.is_some() {
                        SemanticTokenType::METHOD
                    } else {
                        SemanticTokenType::FUNCTION
                    }
                };
                if function_item.uri.eq(&uri) {
                    builder
                        .push(
                            function_item.range,
                            type_.clone(),
                            Some(vec![SemanticTokenModifier::DECLARATION]),
                        )
                        .unwrap();
                }
                for ref_ in function_item.references.iter() {
                    if ref_.uri.eq(&uri) {
                        if function_item.range.eq(&ref_.range) {
                            builder
                                .push(
                                    ref_.range,
                                    type_.clone(),
                                    Some(vec![SemanticTokenModifier::DECLARATION]),
                                )
                                .unwrap();
                        } else {
                            let mut dep: Vec<SemanticTokenModifier> = vec![];
                            if function_item.description.deprecated.is_some() {
                                dep.push(SemanticTokenModifier::DEPRECATED);
                            }
                            builder.push(ref_.range, type_.clone(), Some(dep)).unwrap();
                        }
                    }
                }
            }
            SPItem::Methodmap(mm_item) => {
                if mm_item.uri.eq(&uri) {
                    builder
                        .push(
                            mm_item.range,
                            SemanticTokenType::CLASS,
                            Some(vec![SemanticTokenModifier::DECLARATION]),
                        )
                        .unwrap();
                }
                for ref_ in mm_item.references.iter() {
                    if ref_.uri.eq(&uri) {
                        if mm_item.range.eq(&ref_.range) {
                            builder
                                .push(
                                    ref_.range,
                                    SemanticTokenType::CLASS,
                                    Some(vec![SemanticTokenModifier::DECLARATION]),
                                )
                                .unwrap();
                        } else {
                            builder
                                .push(ref_.range, SemanticTokenType::CLASS, None)
                                .unwrap();
                        }
                    }
                }
            }
            // TODO: Deal with constructors
            _ => {}
        }
    }

    Some(builder.build(None))
}
