use anyhow::Context;
use syntax::utils::ts_range_to_lsp_range;
use tree_sitter::Node;

use crate::{document::Document, include::add_include, Store};

impl Store {
    pub(crate) fn parse_include(
        &mut self,
        document: &mut Document,
        node: &mut Node,
    ) -> anyhow::Result<()> {
        let path_node = node
            .child_by_field_name("path")
            .context("Include path is empty.")?;
        let path = path_node.utf8_text(document.preprocessed_text.as_bytes())?;
        let range = ts_range_to_lsp_range(&path_node.range());

        // Remove leading and trailing "<" and ">" or ".
        if path.len() < 2 {
            // The include path is empty.
            return Ok(());
        }
        let quoted = path.starts_with('"');
        let mut path = path[1..path.len() - 1].trim().to_string();
        match self.resolve_import(&mut path, &document.uri, quoted) {
            Some(uri) => {
                add_include(document, uri, path, range);
            }
            None => {
                document.missing_includes.insert(path, range);
            }
        }

        Ok(())
    }
}
