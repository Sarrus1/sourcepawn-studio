use lsp_types::{SemanticTokenModifier, SemanticTokenType};
use syntax::enum_member_item::EnumMemberItem;
use vfs::FileId;

use super::SemanticTokensBuilder;

impl SemanticTokensBuilder {
    pub(crate) fn build_enum_member(
        &mut self,
        enum_member_item: &EnumMemberItem,
        file_id: FileId,
    ) -> anyhow::Result<()> {
        if enum_member_item.file_id == file_id {
            self.push(
                enum_member_item.v_range,
                SemanticTokenType::ENUM_MEMBER,
                Some(vec![
                    SemanticTokenModifier::READONLY,
                    SemanticTokenModifier::DECLARATION,
                ]),
            )?;
        }
        for reference in enum_member_item.references.iter() {
            if reference.file_id == file_id {
                self.push(
                    reference.v_range,
                    SemanticTokenType::ENUM_MEMBER,
                    Some(vec![SemanticTokenModifier::READONLY]),
                )?;
            }
        }

        Ok(())
    }
}
