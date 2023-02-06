use std::sync::Arc;

use lsp_types::{SemanticTokenModifier, SemanticTokenType, Url};

use crate::spitem::{enum_item::EnumItem, SPItem};

use super::SemanticTokensBuilder;

impl SemanticTokensBuilder {
    pub(crate) fn build_enum(
        &mut self,
        enum_item: &EnumItem,
        uri: &Arc<Url>,
    ) -> anyhow::Result<()> {
        if enum_item.uri.eq(uri) {
            self.push(
                enum_item.range,
                SemanticTokenType::ENUM,
                Some(vec![SemanticTokenModifier::DECLARATION]),
            )?;
        }
        for ref_ in enum_item.references.iter() {
            if ref_.uri.eq(uri) {
                self.push(ref_.range, SemanticTokenType::ENUM, None)?;
            }
        }
        enum_item.children.iter().for_each(|child| {
            if let SPItem::EnumMember(enum_member_item) = &*child.read().unwrap() {
                self.build_enum_member(enum_member_item, uri)
                    .unwrap_or_default();
            }
        });

        Ok(())
    }
}
