use std::sync::Arc;

use lsp_types::{SemanticTokenModifier, SemanticTokenType, Url};

use crate::spitem::variable_item::VariableItem;

use super::SemanticTokensBuilder;

impl SemanticTokensBuilder {
    pub(crate) fn build_es_field(
        &mut self,
        field_item: &VariableItem,
        uri: &Arc<Url>,
    ) -> anyhow::Result<()> {
        if field_item.uri.eq(uri) {
            self.push(
                field_item.range,
                SemanticTokenType::PROPERTY,
                Some(vec![SemanticTokenModifier::DECLARATION]),
            )?;
        }
        for ref_ in field_item.references.iter() {
            if ref_.uri.eq(uri) {
                self.push(ref_.range, SemanticTokenType::PROPERTY, Some(vec![]))?;
            }
        }

        Ok(())
    }
}
