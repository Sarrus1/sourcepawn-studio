use lsp_types::{SemanticTokenModifier, SemanticTokenType};
use path_interner::FileId;
use syntax::{function_item::FunctionItem, SPItem};

use super::SemanticTokensBuilder;

impl SemanticTokensBuilder {
    pub(crate) fn build_method(
        &mut self,
        method_item: &FunctionItem,
        file_id: FileId,
        methodmap_name: &str,
    ) -> anyhow::Result<()> {
        let mut token_type = SemanticTokenType::METHOD;
        if methodmap_name == method_item.name {
            token_type = SemanticTokenType::CLASS
        }
        if method_item.file_id == file_id {
            self.push(
                method_item.v_range,
                token_type.clone(),
                Some(vec![SemanticTokenModifier::DECLARATION]),
            )?;
        }
        for reference in method_item.references.iter() {
            if reference.file_id == file_id {
                self.push(reference.v_range, token_type.clone(), Some(vec![]))?;
            }
        }
        method_item.children.iter().for_each(|child| {
            if let SPItem::Variable(variable_item) = &*child.read() {
                self.build_local_variable(variable_item, file_id)
                    .unwrap_or_default();
            }
        });

        Ok(())
    }
}
