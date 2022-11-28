use std::{
    collections::{HashMap, HashSet},
    str::Utf8Error,
    sync::Arc,
};

use derive_new::new;
use lazy_static::lazy_static;
use lsp_types::{MarkupContent, Url};
use regex::Regex;
use tree_sitter::{Node, Parser};

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
        let mut comments: Vec<Node> = vec![];
        let mut deprecated: Vec<Node> = vec![];
        let mut cursor = root_node.walk();

        for mut node in root_node.children(&mut cursor) {
            let kind = node.kind();
            match kind {
                "function_declaration" | "function_definition" => {
                    parse_function(self, &mut node, &mut comments, &mut deprecated)?;
                }
                "global_variable_declaration" | "old_global_variable_declaration" => {
                    parse_variable(self, &mut node, None)?;
                }
                "preproc_include" | "preproc_tryinclude" => {
                    parse_include(environment, documents, self, &mut node)?;
                }
                "comment" => {
                    comments.push(node);
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

#[derive(Debug, Clone, Default)]
pub struct Description {
    pub text: String,
    pub deprecated: Option<String>,
}

impl Description {
    fn documentation_to_md(&self) -> String {
        lazy_static! {
            static ref RE1: Regex = Regex::new(r"^\*<").unwrap();
            static ref RE2: Regex = Regex::new(r"\*\s*\r?\n\s*\*").unwrap();
            static ref RE3: Regex = Regex::new(r"\r?\n\s*\*").unwrap();
            static ref RE4: Regex = Regex::new(r"^\*").unwrap();
            static ref RE5: Regex = Regex::new(r"<").unwrap();
            static ref RE6: Regex = Regex::new(r">").unwrap();
            static ref RE7: Regex = Regex::new(r"\s*(@[A-Za-z]+)\s+").unwrap();
            static ref RE8: Regex = Regex::new(r"(_@param_) ([A-Za-z0-9_.]+)\s*").unwrap();
            static ref RE9: Regex = Regex::new(r"(\w+\([A-Za-z0-9_ :]*\))").unwrap();
        }
        let text = RE1.replace_all(&self.text, "").into_owned();
        let text = RE2.replace_all(&text, "\n").into_owned();
        let text = RE3.replace_all(&text, "").into_owned();
        let text = RE4.replace_all(&text, "").into_owned();
        let text = RE5.replace_all(&text, "\\<").into_owned();
        let text = RE6.replace_all(&text, "\\>").into_owned();

        let text = RE7.replace_all(&text, "\n\n_${1}_ ").into_owned();
        let text = RE8.replace_all(&text, "${1} `${2}` — >").into_owned();
        let text = RE9.replace_all(&text, "`${1}`").into_owned();
        let text = text.replace("DEPRECATED", "\n\n**DEPRECATED**");

        return text;
    }

    pub fn description_to_md(&self) -> MarkupContent {
        MarkupContent {
            kind: lsp_types::MarkupKind::Markdown,
            value: self.documentation_to_md(),
        }
    }
}

pub fn find_doc(
    comments: &mut Vec<Node>,
    deprecated: &mut Vec<Node>,
    mut end_row: usize,
    source: &String,
) -> Result<Description, Utf8Error> {
    let mut dep: Option<String> = None;
    let mut text: Vec<String> = vec![];

    for deprecated in deprecated.iter().rev() {
        if end_row == deprecated.end_position().row {
            dep = Some(
                deprecated
                    .child_by_field_name("info")
                    .unwrap()
                    .utf8_text(source.as_bytes())?
                    .to_string(),
            );
            break;
        }
        if end_row > deprecated.end_position().row {
            break;
        }
    }
    let mut offset = 1;
    if dep.is_some() {
        offset = 2;
    }

    for comment in comments.iter().rev() {
        if end_row == comment.end_position().row + offset {
            let comment_text = comment.utf8_text(source.as_bytes())?.to_string();
            text.push(comment_to_doc(&comment_text));
            end_row = comment.start_position().row;
        } else {
            break;
        }
    }
    comments.clear();
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
