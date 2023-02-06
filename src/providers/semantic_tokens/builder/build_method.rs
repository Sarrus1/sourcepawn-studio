use std::sync::Arc;

use lsp_types::{SemanticTokenModifier, SemanticTokenType, Url};

use crate::spitem::{function_item::FunctionItem, SPItem};

use super::SemanticTokensBuilder;

impl SemanticTokensBuilder {
    pub(crate) fn build_method(
        &mut self,
        method_item: &FunctionItem,
        uri: &Arc<Url>,
        methodmap_name: &str,
    ) -> anyhow::Result<()> {
        let mut token_type = SemanticTokenType::METHOD;
        if methodmap_name == method_item.name {
            token_type = SemanticTokenType::CLASS
        }
        if method_item.uri.eq(uri) {
            self.push(
                method_item.range,
                token_type.clone(),
                Some(vec![SemanticTokenModifier::DECLARATION]),
            )?;
        }
        for ref_ in method_item.references.iter() {
            if ref_.uri.eq(uri) {
                self.push(ref_.range, token_type.clone(), Some(vec![]))?;
            }
        }
        method_item.children.iter().for_each(|child| {
            if let SPItem::Variable(variable_item) = &*child.read().unwrap() {
                self.build_local_variable(variable_item, uri)
                    .unwrap_or_default();
            }
        });

        Ok(())
    }
}
