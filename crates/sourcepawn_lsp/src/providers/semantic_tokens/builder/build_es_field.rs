use lsp_types::{SemanticTokenModifier, SemanticTokenType};
use syntax::{variable_item::VariableItem, FileId};

use super::SemanticTokensBuilder;

impl SemanticTokensBuilder {
    pub(crate) fn build_es_field(
        &mut self,
        field_item: &VariableItem,
        file_id: FileId,
    ) -> anyhow::Result<()> {
        if field_item.file_id == file_id {
            self.push(
                field_item.v_range,
                SemanticTokenType::PROPERTY,
                Some(vec![SemanticTokenModifier::DECLARATION]),
            )?;
        }
        for reference in field_item.references.iter() {
            if reference.file_id == file_id {
                self.push(reference.v_range, SemanticTokenType::PROPERTY, Some(vec![]))?;
            }
        }

        Ok(())
    }
}
