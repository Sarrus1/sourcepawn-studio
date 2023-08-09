use anyhow::{bail, Context};
use syntax::utils::ts_range_to_lsp_range;
use tree_sitter::Node;

use crate::Parser;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Include {
    pub path: String,
    pub range: lsp_types::Range,
    pub quoted: bool,
}

impl<'a> Parser<'a> {
    pub fn parse_include(&mut self, node: &mut Node) -> anyhow::Result<Include> {
        let path_node = node
            .child_by_field_name("path")
            .context("Include path is empty.")?;
        let path = path_node.utf8_text(self.source.as_bytes())?;
        let range = ts_range_to_lsp_range(&path_node.range());

        // Remove leading and trailing "<" and ">" or ".
        if path.len() < 2 {
            // The include path is empty.
            bail!("Include path is empty.");
        }
        let quoted = path.starts_with('"');

        Ok(Include {
            path: path[1..path.len() - 1].trim().to_string(),
            range,
            quoted,
        })
    }
}
