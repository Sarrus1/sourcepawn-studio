use lsp_types::{SemanticTokenModifier, SemanticTokenType, Url};

use crate::spitem::{enum_item::EnumItem, SPItem};

use super::SemanticTokensBuilder;

impl SemanticTokensBuilder {
    pub(crate) fn build_enum(&mut self, enum_item: &EnumItem, uri: &Url) -> anyhow::Result<()> {
        if *enum_item.uri == *uri {
            self.push(
                enum_item.v_range,
                SemanticTokenType::ENUM,
                Some(vec![SemanticTokenModifier::DECLARATION]),
            )?;
        }
        for ref_ in enum_item.references.iter() {
            if *ref_.uri == *uri {
                self.push(ref_.v_range, SemanticTokenType::ENUM, None)?;
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
