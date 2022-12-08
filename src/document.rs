use std::{
    collections::{HashMap, HashSet},
    str::Utf8Error,
    sync::{Arc, Mutex},
};

use derive_new::new;
use lazy_static::lazy_static;
use lsp_types::{Range, Url};
use regex::Regex;
use tree_sitter::{Node, Parser};

use crate::{
    environment::Environment,
    parser::{
        enum_parser::parse_enum, enum_struct_parser::parse_enum_struct,
        function_parser::parse_function, include_parser::parse_include,
        variable_parser::parse_variable,
    },
    providers::hover::description::Description,
    spitem::SPItem,
    utils::ts_range_to_lsp_range,
};

#[derive(Debug, Clone, new)]
pub struct Document {
    pub uri: Arc<Url>,
    pub text: String,
    #[new(default)]
    pub sp_items: Vec<Arc<Mutex<SPItem>>>,
    #[new(default)]
    pub includes: HashSet<Url>,
    #[new(value = "false")]
    pub parsed: bool,
}

pub struct Walker {
    pub comments: Vec<Comment>,
    pub deprecated: Vec<Deprecated>,
    pub anon_enum_counter: u32,
}

pub struct Comment {
    text: String,
    range: Range,
}

impl Comment {
    pub fn new(node: Node, source: &str) -> Self {
        Self {
            text: node.utf8_text(source.as_bytes()).unwrap().to_string(),
            range: ts_range_to_lsp_range(&node.range()),
        }
    }
}

pub struct Deprecated {
    text: String,
    range: Range,
}

impl Deprecated {
    pub fn new(node: Node, source: &str) -> Self {
        Self {
            text: node
                .child_by_field_name("info")
                .unwrap()
                .utf8_text(source.as_bytes())
                .unwrap()
                .to_string(),
            range: ts_range_to_lsp_range(&node.range()),
        }
    }
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
        let mut walker = Walker {
            comments: vec![],
            deprecated: vec![],
            anon_enum_counter: 0,
        };

        let mut cursor = root_node.walk();

        for mut node in root_node.children(&mut cursor) {
            let kind = node.kind();
            match kind {
                "function_declaration" | "function_definition" => {
                    parse_function(self, &mut node, &mut walker, None)?;
                }
                "global_variable_declaration" | "old_global_variable_declaration" => {
                    parse_variable(self, &mut node, None)?;
                }
                "preproc_include" | "preproc_tryinclude" => {
                    parse_include(environment, documents, self, &mut node)?;
                }
                "enum" => {
                    parse_enum(self, &mut node, &mut walker)?;
                }
                "enum_struct" => parse_enum_struct(self, &mut node, &mut walker)?,
                "comment" => {
                    walker.comments.push(Comment::new(node, &self.text));
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

pub fn find_doc(walker: &mut Walker, end_row: usize) -> Result<Description, Utf8Error> {
    let mut end_row = end_row as u32;
    let mut dep: Option<String> = None;
    let mut text: Vec<String> = vec![];

    for deprecated in walker.deprecated.iter().rev() {
        if end_row == deprecated.range.end.line {
            dep = Some(deprecated.text.clone());
            break;
        }
        if end_row > deprecated.range.end.line {
            break;
        }
    }
    let mut offset = 1;
    if dep.is_some() {
        offset = 2;
    }

    for comment in walker.comments.iter().rev() {
        if end_row == comment.range.end.line + offset {
            let comment_text = comment.text.clone();
            text.push(comment_to_doc(&comment_text));
            end_row = comment.range.start.line;
        } else {
            break;
        }
    }
    walker.comments.clear();
    let doc = Description {
        text: text.join(""),
        deprecated: dep,
    };

    Ok(doc)
}

fn comment_to_doc(text: &String) -> String {
    lazy_static! {
        static ref RE1: Regex = Regex::new(r"^/\*\s*").unwrap();
        static ref RE2: Regex = Regex::new(r"\*/$").unwrap();
        static ref RE3: Regex = Regex::new(r"//\s*").unwrap();
    }
    let text = RE1.replace_all(&text, "").into_owned();
    let text = RE2.replace_all(&text, "").into_owned();
    let text = RE3.replace_all(&text, "").into_owned();

    return text;
}
