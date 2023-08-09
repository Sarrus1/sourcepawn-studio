use lsp_types::{SemanticTokenModifier, SemanticTokenType, Url};
use syntax::variable_item::VariableItem;

use super::SemanticTokensBuilder;

impl SemanticTokensBuilder {
    pub(crate) fn build_es_field(
        &mut self,
        field_item: &VariableItem,
        uri: &Url,
    ) -> anyhow::Result<()> {
        if *field_item.uri == *uri {
            self.push(
                field_item.v_range,
                SemanticTokenType::PROPERTY,
                Some(vec![SemanticTokenModifier::DECLARATION]),
            )?;
        }
        for ref_ in field_item.references.iter() {
            if *ref_.uri == *uri {
                self.push(ref_.v_range, SemanticTokenType::PROPERTY, Some(vec![]))?;
            }
        }

        Ok(())
    }
}
