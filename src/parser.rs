use std::str::Utf8Error;

use tree_sitter::Parser;

use crate::fileitem::FileItem;

use self::{function_parser::parse_function, include_parser::parse_include};

mod function_parser;
mod include_parser;

pub fn parse_document(parser: &mut Parser, file_item: &mut FileItem) -> Result<(), Utf8Error> {
    let tree = parser.parse(&file_item.text, None).unwrap();
    let root_node = tree.root_node();
    let mut cursor = root_node.walk();

    for mut node in root_node.children(&mut cursor) {
        let kind = node.kind();
        match kind {
            "function_declaration" | "function_definition" => {
                parse_function(file_item, &mut node)?;
            }
            "preproc_include" | "preproc_tryinclude" => {
                parse_include(file_item, &mut node)?;
            }
            _ => {
                continue;
            }
        }
    }

    Ok(())
}
