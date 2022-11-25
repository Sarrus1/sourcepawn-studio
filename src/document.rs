use std::{
    collections::{HashMap, HashSet},
    str::Utf8Error,
    sync::Arc,
};

use derive_new::new;
use lsp_types::Url;
use tree_sitter::Parser;

use crate::{
    environment::Environment,
    parser::{
        function_parser::parse_function, include_parser::parse_include,
        variable_parser::parse_variable,
    },
    spitem::SPItem,
};

#[derive(Debug, Clone, new)]
pub struct Document {
    pub uri: Arc<Url>,
    pub text: String,
    #[new(default)]
    pub sp_items: Vec<Arc<SPItem>>,
    #[new(default)]
    pub includes: HashSet<Url>,
    #[new(value = "false")]
    pub parsed: bool,
}

impl Document {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn parse(
        &mut self,
        environment: &Environment,
        parser: &mut Parser,
        documents: &HashMap<Arc<Url>, Document>,
    ) -> Result<(), Utf8Error> {
        let tree = parser.parse(&self.text, None).unwrap();
        let root_node = tree.root_node();
        let mut cursor = root_node.walk();

        for mut node in root_node.children(&mut cursor) {
            let kind = node.kind();
            match kind {
                "function_declaration" | "function_definition" => {
                    parse_function(self, &mut node)?;
                }
                "global_variable_declaration" | "old_global_variable_declaration" => {
                    parse_variable(self, &mut node, None)?;
                }
                "preproc_include" | "preproc_tryinclude" => {
                    parse_include(environment, documents, self, &mut node)?;
                }
                _ => {
                    continue;
                }
            }
        }
        self.parsed = true;

        Ok(())
    }
}
