use std::sync::Arc;

use lsp_types::{SemanticTokenModifier, SemanticTokenType, Url};

use crate::spitem::enum_member_item::EnumMemberItem;

use super::SemanticTokensBuilder;

impl SemanticTokensBuilder {
    pub(crate) fn build_enum_member(
        &mut self,
        enum_member_item: &EnumMemberItem,
        uri: &Arc<Url>,
    ) -> anyhow::Result<()> {
        if enum_member_item.uri.eq(uri) {
            self.push(
                enum_member_item.v_range,
                SemanticTokenType::ENUM_MEMBER,
                Some(vec![
                    SemanticTokenModifier::READONLY,
                    SemanticTokenModifier::DECLARATION,
                ]),
            )?;
        }
        for ref_ in enum_member_item.references.iter() {
            if ref_.uri.eq(uri) {
                self.push(
                    ref_.v_range,
                    SemanticTokenType::ENUM_MEMBER,
                    Some(vec![SemanticTokenModifier::READONLY]),
                )?;
            }
        }

        Ok(())
    }
}
