use std::sync::Arc;

use lsp_types::{SemanticTokenModifier, SemanticTokenType, Url};

use crate::spitem::property_item::PropertyItem;

use super::SemanticTokensBuilder;

impl SemanticTokensBuilder {
    pub(crate) fn build_property(
        &mut self,
        property_item: &PropertyItem,
        uri: &Arc<Url>,
    ) -> anyhow::Result<()> {
        if property_item.uri.eq(uri) {
            self.push(
                property_item.range,
                SemanticTokenType::PROPERTY,
                Some(vec![SemanticTokenModifier::DECLARATION]),
            )?;
        }
        for ref_ in property_item.references.iter() {
            if ref_.uri.eq(uri) {
                self.push(ref_.range, SemanticTokenType::PROPERTY, Some(vec![]))?;
            }
        }

        Ok(())
    }
}
