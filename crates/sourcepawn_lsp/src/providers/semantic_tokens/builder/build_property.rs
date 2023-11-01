use lsp_types::{SemanticTokenModifier, SemanticTokenType};
use path_interner::FileId;
use syntax::property_item::PropertyItem;

use super::SemanticTokensBuilder;

impl SemanticTokensBuilder {
    pub(crate) fn build_property(
        &mut self,
        property_item: &PropertyItem,
        file_id: FileId,
    ) -> anyhow::Result<()> {
        if property_item.file_id == file_id {
            self.push(
                property_item.v_range,
                SemanticTokenType::PROPERTY,
                Some(vec![SemanticTokenModifier::DECLARATION]),
            )?;
        }
        for reference in property_item.references.iter() {
            if reference.file_id == file_id {
                self.push(reference.v_range, SemanticTokenType::PROPERTY, Some(vec![]))?;
            }
        }

        Ok(())
    }
}
