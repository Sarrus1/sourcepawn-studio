use lsp_types::{SemanticTokenModifier, SemanticTokenType};
use path_interner::FileId;
use syntax::variable_item::VariableItem;

use super::SemanticTokensBuilder;

impl SemanticTokensBuilder {
    pub(crate) fn build_local_variable(
        &mut self,
        variable_item: &VariableItem,
        file_id: FileId,
    ) -> anyhow::Result<()> {
        if variable_item.file_id == file_id {
            self.push(
                variable_item.v_range,
                SemanticTokenType::VARIABLE,
                Some(vec![SemanticTokenModifier::DECLARATION]),
            )?;
        }
        for reference in variable_item.references.iter() {
            if reference.file_id == file_id {
                self.push(reference.v_range, SemanticTokenType::VARIABLE, Some(vec![]))?;
            }
        }

        Ok(())
    }
}
