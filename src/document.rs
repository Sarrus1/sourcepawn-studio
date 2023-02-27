use std::{
    collections::HashSet,
    str::Utf8Error,
    sync::{Arc, RwLock},
};

use derive_new::new;
use lazy_static::lazy_static;
use lsp_types::Range;
use lsp_types::Url;
use regex::Regex;
use tree_sitter::{Node, Parser, Query, QueryCursor};

use crate::{
    parser::{
        comment_parser::{Comment, Deprecated},
        define_parser::parse_define,
        enum_parser::parse_enum,
        enum_struct_parser::parse_enum_struct,
        function_parser::parse_function,
        include_parser::parse_include,
        methodmap_parser::parse_methodmap,
        variable_parser::parse_variable,
    },
    providers::hover::description::Description,
    spitem::SPItem,
    store::Store,
    utils::ts_range_to_lsp_range,
};

lazy_static! {
    static ref SYMBOL_QUERY: Query = {
        Query::new(
            tree_sitter_sourcepawn::language(),
            "[(symbol) @symbol (this) @symbol]",
        )
        .unwrap()
    };
}

#[derive(Debug, Clone)]
pub struct Token {
    pub text: String,
    pub range: Range,
}

impl Token {
    pub fn new(node: Node, source: &String) -> Self {
        Self {
            text: node.utf8_text(source.as_bytes()).unwrap().to_string(),
            range: ts_range_to_lsp_range(&node.range()),
        }
    }
}

#[derive(Debug, Clone, new)]
pub struct Document {
    pub uri: Arc<Url>,
    pub text: String,
    #[new(default)]
    pub sp_items: Vec<Arc<RwLock<SPItem>>>,
    #[new(default)]
    pub includes: HashSet<Url>,
    #[new(value = "false")]
    pub parsed: bool,
    #[new(value = "vec![]")]
    pub tokens: Vec<Arc<Token>>,
}

pub struct Walker {
    pub comments: Vec<Comment>,
    pub deprecated: Vec<Deprecated>,
    pub anon_enum_counter: u32,
}

impl Document {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn parse(&mut self, store: &mut Store, parser: &mut Parser) -> Result<(), Utf8Error> {
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
                    parse_function(self, &node, &mut walker, None)?;
                }
                "global_variable_declaration" | "old_global_variable_declaration" => {
                    parse_variable(self, &mut node, None)?;
                }
                "preproc_include" | "preproc_tryinclude" => {
                    parse_include(store, self, &mut node)?;
                }
                "enum" => {
                    parse_enum(self, &mut node, &mut walker)?;
                }
                "preproc_define" => {
                    parse_define(self, &mut node, &mut walker)?;
                }
                "methodmap" => {
                    parse_methodmap(self, &mut node, &mut walker)?;
                }
                "typedef" => {}
                "typeset" => {}
                "preproc_macro" => {}
                "enum_struct" => parse_enum_struct(self, &mut node, &mut walker)?,
                "comment" => {
                    walker.push_comment(node, &self.text);
                }
                "preproc_pragma" => walker.push_deprecated(node, &self.text),
                _ => {
                    continue;
                }
            }
        }
        self.parsed = true;
        self.extract_tokens(root_node);
        store.documents.insert(self.uri.clone(), self.clone());
        store.read_unscanned_imports(&self.includes, parser);

        Ok(())
    }

    pub fn extract_tokens(&mut self, root_node: Node) {
        let mut cursor = QueryCursor::new();
        let matches = cursor.captures(&SYMBOL_QUERY, root_node, self.text.as_bytes());
        for (match_, _) in matches {
            for capture in match_.captures.iter() {
                self.tokens
                    .push(Arc::new(Token::new(capture.node, &self.text)));
            }
        }
    }

    pub fn line(&self, line_nb: u32) -> Option<&str> {
        for (i, line) in self.text.lines().enumerate() {
            if i == line_nb as usize {
                return Some(line);
            }
        }

        None
    }
}

pub fn find_doc(walker: &mut Walker, end_row: usize) -> Result<Description, Utf8Error> {
    let mut end_row = end_row as u32;
    let mut dep: Option<String> = None;
    let mut text: Vec<String> = vec![];

    for deprecated in walker.deprecated.iter().rev() {
        if end_row == deprecated.range.end.line + 1 {
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

fn comment_to_doc(text: &str) -> String {
    lazy_static! {
        static ref RE1: Regex = Regex::new(r"^/\*\s*").unwrap();
        static ref RE2: Regex = Regex::new(r"\*/$").unwrap();
        static ref RE3: Regex = Regex::new(r"//\s*").unwrap();
    }
    let text = RE1.replace_all(text, "").into_owned();
    let text = RE2.replace_all(&text, "").into_owned();
    let text = RE3.replace_all(&text, "").into_owned();

    text
}
