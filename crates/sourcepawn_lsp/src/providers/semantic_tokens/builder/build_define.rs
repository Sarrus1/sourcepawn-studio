use lsp_types::{SemanticTokenModifier, SemanticTokenType, Url};
use syntax::define_item::DefineItem;

use super::SemanticTokensBuilder;

impl SemanticTokensBuilder {
    pub(crate) fn build_define(
        &mut self,
        define_item: &DefineItem,
        uri: &Url,
    ) -> anyhow::Result<()> {
        if *define_item.uri == *uri {
            self.push(
                define_item.v_range,
                SemanticTokenType::MACRO,
                Some(vec![
                    SemanticTokenModifier::READONLY,
                    SemanticTokenModifier::DECLARATION,
                ]),
            )?;
        }
        for ref_ in define_item.references.iter() {
            if *ref_.uri == *uri {
                self.push(
                    ref_.v_range,
                    SemanticTokenType::MACRO,
                    Some(vec![SemanticTokenModifier::READONLY]),
                )?;
            }
        }

        Ok(())
    }
}
