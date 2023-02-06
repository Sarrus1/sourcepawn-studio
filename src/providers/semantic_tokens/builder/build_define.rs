use std::sync::Arc;

use lsp_types::{SemanticTokenModifier, SemanticTokenType, Url};

use crate::spitem::define_item::DefineItem;

use super::SemanticTokensBuilder;

impl SemanticTokensBuilder {
    pub(crate) fn build_define(
        &mut self,
        define_item: &DefineItem,
        uri: &Arc<Url>,
    ) -> anyhow::Result<()> {
        if define_item.uri.eq(uri) {
            self.push(
                define_item.range,
                SemanticTokenType::MACRO,
                Some(vec![
                    SemanticTokenModifier::READONLY,
                    SemanticTokenModifier::DECLARATION,
                ]),
            )?;
        }
        for ref_ in define_item.references.iter() {
            if ref_.uri.eq(uri) {
                self.push(
                    ref_.range,
                    SemanticTokenType::MACRO,
                    Some(vec![SemanticTokenModifier::READONLY]),
                )?;
            }
        }

        Ok(())
    }
}
