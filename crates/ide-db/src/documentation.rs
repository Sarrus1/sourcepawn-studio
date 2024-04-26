use lazy_static::lazy_static;
use regex::Regex;
use syntax::TSKind;

/// Holds documentation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Documentation(String);

impl From<Documentation> for String {
    fn from(Documentation(string): Documentation) -> Self {
        string
    }
}

impl Documentation {
    pub fn new(s: String) -> Self {
        Documentation(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn from_node(mut node: tree_sitter::Node, source: &[u8]) -> Option<Documentation> {
        let mut pragma = None;
        let mut docs = Vec::new();
        match TSKind::from(&node) {
            TSKind::preproc_define => {
                if let Some(prev_node) = node.prev_sibling() {
                    if TSKind::from(prev_node) == TSKind::preproc_pragma {
                        pragma = Some(
                            prev_node
                                .utf8_text(source)
                                .ok()?
                                .trim_start_matches("#pragma deprecated")
                                .trim(),
                        );
                    }
                }
                while let Some(next_node) = node.next_sibling() {
                    if node.range().end_point.row != next_node.range().start_point.row {
                        break;
                    }
                    if TSKind::from(next_node) != TSKind::comment {
                        break;
                    }
                    docs.push(comment_to_doc(next_node.utf8_text(source).ok()?));
                    if next_node.range().start_point.row != next_node.range().end_point.row {
                        // Only keep one multi-line comment
                        break;
                    }
                    node = next_node;
                }
            }
            TSKind::function_declaration
            | TSKind::function_definition
            | TSKind::typedef
            | TSKind::typeset
            | TSKind::functag
            | TSKind::funcenum
            | TSKind::r#enum
            | TSKind::enum_struct
            | TSKind::enum_struct_field
            | TSKind::enum_struct_method
            | TSKind::methodmap
            | TSKind::methodmap_alias
            | TSKind::methodmap_method
            | TSKind::methodmap_method_constructor
            | TSKind::methodmap_method_destructor
            | TSKind::methodmap_property_getter
            | TSKind::methodmap_property_setter
            | TSKind::methodmap_property_native
            | TSKind::methodmap_property_method
            | TSKind::methodmap_native
            | TSKind::methodmap_native_constructor
            | TSKind::methodmap_native_destructor => {
                while let Some(prev_node) = node.prev_sibling() {
                    if node
                        .range()
                        .start_point
                        .row
                        .saturating_sub(prev_node.range().end_point.row)
                        != 1
                    {
                        break;
                    }
                    if TSKind::from(prev_node) == TSKind::preproc_pragma {
                        if pragma.is_some() {
                            break;
                        }
                        pragma = Some(
                            prev_node
                                .utf8_text(source)
                                .ok()?
                                .trim_start_matches("#pragma deprecated")
                                .trim(),
                        );
                        node = prev_node;
                        continue;
                    }
                    if TSKind::from(prev_node) != TSKind::comment {
                        break;
                    }
                    docs.push(comment_to_doc(prev_node.utf8_text(source).ok()?));
                    if prev_node.range().start_point.row != prev_node.range().end_point.row {
                        // Only keep one multi-line comment
                        break;
                    }
                    node = prev_node;
                }
            }
            _ => (),
        }
        if pragma.is_none() && docs.is_empty() {
            return None;
        }
        if let Some(pragma) = pragma {
            docs.push(format!("DEPRECATED: {}\n", pragma));
        }
        docs.reverse();
        Documentation::new(docs.join("\n")).into()
    }

    pub fn to_markdown(&self) -> String {
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
        let text = RE1.replace_all(self.as_str(), "").into_owned();
        let text = RE2.replace_all(&text, "\n\n").into_owned();
        let text = RE3.replace_all(&text, "").into_owned();
        let text = RE4.replace_all(&text, "").into_owned();
        let text = RE5.replace_all(&text, "\\<").into_owned();
        let text = RE6.replace_all(&text, "\\>").into_owned();

        let text = RE7.replace_all(&text, "\n\n_${1}_ ").into_owned();
        let text = RE8.replace_all(&text, "${1} `${2}` — >").into_owned();
        let text = RE9.replace_all(&text, "`${1}`").into_owned();
        text.replace("DEPRECATED", "\n\n**DEPRECATED**")
    }
}

fn comment_to_doc(text: &str) -> String {
    lazy_static! {
        static ref RE1: Regex = Regex::new(r"^\s*/(?:\*)+\s*").unwrap();
        static ref RE2: Regex = Regex::new(r"\*/$").unwrap();
        static ref RE3: Regex = Regex::new(r"^\s*//\s*").unwrap();
    }
    let text = RE1.replace_all(text, "").into_owned();
    let text = RE2.replace_all(&text, "").into_owned();
    let text = RE3.replace_all(&text, "").into_owned();

    text
}
