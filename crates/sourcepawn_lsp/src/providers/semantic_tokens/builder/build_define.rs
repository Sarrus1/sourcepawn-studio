use lsp_types::{SemanticTokenModifier, SemanticTokenType};
use syntax::define_item::DefineItem;
use vfs::FileId;

use super::SemanticTokensBuilder;

impl SemanticTokensBuilder {
    pub(crate) fn build_define(
        &mut self,
        define_item: &DefineItem,
        file_id: FileId,
    ) -> anyhow::Result<()> {
        if define_item.file_id == file_id {
            self.push(
                define_item.v_range,
                SemanticTokenType::MACRO,
                Some(vec![
                    SemanticTokenModifier::READONLY,
                    SemanticTokenModifier::DECLARATION,
                ]),
            )?;
        }
        for reference in define_item.references.iter() {
            if reference.file_id == file_id {
                self.push(
                    reference.v_range,
                    SemanticTokenType::MACRO,
                    Some(vec![SemanticTokenModifier::READONLY]),
                )?;
            }
        }

        Ok(())
    }
}
