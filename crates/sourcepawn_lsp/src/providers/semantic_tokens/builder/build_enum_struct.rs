use lsp_types::{SemanticTokenModifier, SemanticTokenType, Url};
use syntax::{enum_struct_item::EnumStructItem, SPItem};

use super::SemanticTokensBuilder;

impl SemanticTokensBuilder {
    pub(crate) fn build_enum_struct(
        &mut self,
        es_item: &EnumStructItem,
        uri: &Url,
    ) -> anyhow::Result<()> {
        if *es_item.uri == *uri {
            self.push(
                es_item.v_range,
                SemanticTokenType::STRUCT,
                Some(vec![SemanticTokenModifier::DECLARATION]),
            )?;
        }
        for ref_ in es_item.references.iter() {
            if *ref_.uri == *uri {
                self.push(ref_.v_range, SemanticTokenType::STRUCT, None)?;
            }
        }
        es_item.children.iter().for_each(|child| {
            match &*child.read() {
                SPItem::Function(method_item) => self.build_method(method_item, uri, ""),
                SPItem::Variable(es_field) => self.build_es_field(es_field, uri),
                _ => Ok(()),
            }
            .unwrap_or_default();
        });

        Ok(())
    }
}
