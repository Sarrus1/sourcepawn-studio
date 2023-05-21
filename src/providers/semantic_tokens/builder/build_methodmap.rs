use std::sync::Arc;

use lsp_types::{SemanticTokenModifier, SemanticTokenType, Url};

use crate::spitem::{methodmap_item::MethodmapItem, SPItem};

use super::SemanticTokensBuilder;

impl SemanticTokensBuilder {
    pub(crate) fn build_methodmap(
        &mut self,
        mm_item: &MethodmapItem,
        uri: &Arc<Url>,
    ) -> anyhow::Result<()> {
        if mm_item.uri.eq(uri) {
            self.push(
                mm_item.v_range,
                SemanticTokenType::CLASS,
                Some(vec![SemanticTokenModifier::DECLARATION]),
            )?;
        }
        for ref_ in mm_item.references.iter() {
            if ref_.uri.eq(uri) {
                self.push(ref_.v_range, SemanticTokenType::CLASS, None)?;
            }
        }
        mm_item.children.iter().for_each(|child| {
            match &*child.read().unwrap() {
                SPItem::Function(method_item) => self.build_method(method_item, uri, &mm_item.name),
                SPItem::Property(property_item) => self.build_property(property_item, uri),
                _ => Ok(()),
            }
            .unwrap_or_default();
        });

        Ok(())
    }
}
