use std::{
    collections::{HashMap, HashSet},
    str::Utf8Error,
};

use derive_new::new;
use tree_sitter::Parser;

use crate::{
    environment::Environment,
    parser::{function_parser::parse_function, include_parser::parse_include},
    spitem::SPItem,
};

#[derive(Debug, Default, Clone, new)]
pub struct Document {
    pub uri: String,
    pub text: String,
    pub sp_items: Vec<SPItem>,
    pub includes: HashSet<String>,
}

impl Document {
    pub fn parse(
        &mut self,
        environment: &Environment,
        parser: &mut Parser,
        documents: &HashMap<String, Document>,
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
                "preproc_include" | "preproc_tryinclude" => {
                    parse_include(environment, documents, self, &mut node)?;
                }
                _ => {
                    continue;
                }
            }
        }

        Ok(())
    }
}
