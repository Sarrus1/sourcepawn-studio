use lsp_types::{SemanticTokenModifier, SemanticTokenType};
use syntax::{enum_item::EnumItem, SPItem};
use vfs::FileId;

use super::SemanticTokensBuilder;

impl SemanticTokensBuilder {
    pub(crate) fn build_enum(
        &mut self,
        enum_item: &EnumItem,
        file_id: FileId,
    ) -> anyhow::Result<()> {
        if enum_item.file_id == file_id {
            self.push(
                enum_item.v_range,
                SemanticTokenType::ENUM,
                Some(vec![SemanticTokenModifier::DECLARATION]),
            )?;
        }
        for reference in enum_item.references.iter() {
            if reference.file_id == file_id {
                self.push(reference.v_range, SemanticTokenType::ENUM, None)?;
            }
        }
        enum_item.children.iter().for_each(|child| {
            if let SPItem::EnumMember(enum_member_item) = &*child.read() {
                self.build_enum_member(enum_member_item, file_id)
                    .unwrap_or_default();
            }
        });

        Ok(())
    }
}
