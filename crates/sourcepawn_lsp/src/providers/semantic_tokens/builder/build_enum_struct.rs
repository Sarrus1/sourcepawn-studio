use lsp_types::{SemanticTokenModifier, SemanticTokenType};
use path_interner::FileId;
use syntax::{enum_struct_item::EnumStructItem, SPItem};

use super::SemanticTokensBuilder;

impl SemanticTokensBuilder {
    pub(crate) fn build_enum_struct(
        &mut self,
        es_item: &EnumStructItem,
        file_id: FileId,
    ) -> anyhow::Result<()> {
        if es_item.file_id == file_id {
            self.push(
                es_item.v_range,
                SemanticTokenType::STRUCT,
                Some(vec![SemanticTokenModifier::DECLARATION]),
            )?;
        }
        for reference in es_item.references.iter() {
            if reference.file_id == file_id {
                self.push(reference.v_range, SemanticTokenType::STRUCT, None)?;
            }
        }
        es_item.children.iter().for_each(|child| {
            match &*child.read() {
                SPItem::Function(method_item) => self.build_method(method_item, file_id, ""),
                SPItem::Variable(es_field) => self.build_es_field(es_field, file_id),
                _ => Ok(()),
            }
            .unwrap_or_default();
        });

        Ok(())
    }
}
