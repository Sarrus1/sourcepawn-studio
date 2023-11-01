use lsp_types::{SemanticTokenModifier, SemanticTokenType};
use path_interner::FileId;
use syntax::{function_item::FunctionItem, SPItem};

use super::SemanticTokensBuilder;

impl SemanticTokensBuilder {
    pub(crate) fn build_function(
        &mut self,
        function_item: &FunctionItem,
        file_id: FileId,
    ) -> anyhow::Result<()> {
        let type_ = {
            if function_item.parent.is_some() {
                SemanticTokenType::METHOD
            } else {
                SemanticTokenType::FUNCTION
            }
        };
        if function_item.file_id == file_id {
            self.push(
                function_item.v_range,
                type_.clone(),
                Some(vec![SemanticTokenModifier::DECLARATION]),
            )?;
        }
        for reference in function_item.references.iter() {
            if reference.file_id == file_id {
                let mut modifiers = vec![];
                if function_item.v_range.eq(&reference.v_range) {
                    modifiers.push(SemanticTokenModifier::DECLARATION);
                }
                if function_item.description.deprecated.is_some() {
                    modifiers.push(SemanticTokenModifier::DEPRECATED);
                }
                self.push(reference.v_range, type_.clone(), Some(modifiers))?;
            }
        }
        function_item.children.iter().for_each(|child| {
            if let SPItem::Variable(variable_item) = &*child.read() {
                self.build_local_variable(variable_item, file_id)
                    .unwrap_or_default();
            }
        });

        Ok(())
    }
}
