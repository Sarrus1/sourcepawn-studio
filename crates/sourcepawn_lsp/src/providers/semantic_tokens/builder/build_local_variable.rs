use lsp_types::{SemanticTokenModifier, SemanticTokenType, Url};

use crate::spitem::variable_item::VariableItem;

use super::SemanticTokensBuilder;

impl SemanticTokensBuilder {
    pub(crate) fn build_local_variable(
        &mut self,
        variable_item: &VariableItem,
        uri: &Url,
    ) -> anyhow::Result<()> {
        if *variable_item.uri == *uri {
            self.push(
                variable_item.v_range,
                SemanticTokenType::VARIABLE,
                Some(vec![SemanticTokenModifier::DECLARATION]),
            )?;
        }
        for ref_ in variable_item.references.iter() {
            if *ref_.uri == *uri {
                self.push(ref_.v_range, SemanticTokenType::VARIABLE, Some(vec![]))?;
            }
        }

        Ok(())
    }
}
