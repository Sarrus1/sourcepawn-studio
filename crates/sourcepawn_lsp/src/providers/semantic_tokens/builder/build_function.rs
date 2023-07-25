use std::sync::Arc;

use lsp_types::{SemanticTokenModifier, SemanticTokenType, Url};

use crate::spitem::{function_item::FunctionItem, SPItem};

use super::SemanticTokensBuilder;

impl SemanticTokensBuilder {
    pub(crate) fn build_function(
        &mut self,
        function_item: &FunctionItem,
        uri: &Arc<Url>,
    ) -> anyhow::Result<()> {
        let type_ = {
            if function_item.parent.is_some() {
                SemanticTokenType::METHOD
            } else {
                SemanticTokenType::FUNCTION
            }
        };
        if function_item.uri.eq(uri) {
            self.push(
                function_item.v_range,
                type_.clone(),
                Some(vec![SemanticTokenModifier::DECLARATION]),
            )?;
        }
        for ref_ in function_item.references.iter() {
            if ref_.uri.eq(uri) {
                let mut modifiers = vec![];
                if function_item.v_range.eq(&ref_.v_range) {
                    modifiers.push(SemanticTokenModifier::DECLARATION);
                }
                if function_item.description.deprecated.is_some() {
                    modifiers.push(SemanticTokenModifier::DEPRECATED);
                }
                self.push(ref_.v_range, type_.clone(), Some(modifiers))?;
            }
        }
        function_item.children.iter().for_each(|child| {
            if let SPItem::Variable(variable_item) = &*child.read().unwrap() {
                self.build_local_variable(variable_item, uri)
                    .unwrap_or_default();
            }
        });

        Ok(())
    }
}
