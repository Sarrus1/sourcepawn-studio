use lsp_types::{SemanticTokenModifier, SemanticTokenType};
use path_interner::FileId;
use syntax::{methodmap_item::MethodmapItem, SPItem};

use super::SemanticTokensBuilder;

impl SemanticTokensBuilder {
    pub(crate) fn build_methodmap(
        &mut self,
        mm_item: &MethodmapItem,
        file_id: FileId,
    ) -> anyhow::Result<()> {
        if mm_item.file_id == file_id {
            self.push(
                mm_item.v_range,
                SemanticTokenType::CLASS,
                Some(vec![SemanticTokenModifier::DECLARATION]),
            )?;
        }
        for reference in mm_item.references.iter() {
            if reference.file_id == file_id {
                self.push(reference.v_range, SemanticTokenType::CLASS, None)?;
            }
        }
        mm_item.children.iter().for_each(|child| {
            match &*child.read() {
                SPItem::Function(method_item) => {
                    self.build_method(method_item, file_id, &mm_item.name)
                }
                SPItem::Property(property_item) => self.build_property(property_item, file_id),
                _ => Ok(()),
            }
            .unwrap_or_default();
        });

        Ok(())
    }
}
