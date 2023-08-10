use lsp_types::{SemanticTokenModifier, SemanticTokenType, Url};
use syntax::property_item::PropertyItem;

use super::SemanticTokensBuilder;

impl SemanticTokensBuilder {
    pub(crate) fn build_property(
        &mut self,
        property_item: &PropertyItem,
        uri: &Url,
    ) -> anyhow::Result<()> {
        if *property_item.uri == *uri {
            self.push(
                property_item.v_range,
                SemanticTokenType::PROPERTY,
                Some(vec![SemanticTokenModifier::DECLARATION]),
            )?;
        }
        for ref_ in property_item.references.iter() {
            if *ref_.uri == *uri {
                self.push(ref_.v_range, SemanticTokenType::PROPERTY, Some(vec![]))?;
            }
        }

        Ok(())
    }
}
