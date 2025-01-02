use completion_data::Event;
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

impl From<&str> for Documentation {
    fn from(s: &str) -> Self {
        Documentation(s.to_string())
    }
}

impl From<&Event<'_>> for Documentation {
    fn from(event: &Event) -> Self {
        let mut docs = Vec::new();
        if let Some(note) = event.note() {
            docs.push(note.to_string());
            docs.push("---".to_string());
        }
        for attr in event.attributes() {
            let mut buf = format!("`{}` (__{}__)", attr.name(), attr.r#type(),);
            if let Some(desc) = attr.description() {
                buf.push_str(&format!(" — {}", desc));
            }
            docs.push(buf);
        }
        Documentation(docs.join("\n\n"))
    }
}

impl From<Documentation> for lsp_types::Documentation {
    fn from(val: Documentation) -> Self {
        lsp_types::Documentation::MarkupContent(lsp_types::MarkupContent {
            kind: lsp_types::MarkupKind::Markdown,
            value: val.to_markdown(),
        })
    }
}

impl From<String> for Documentation {
    fn from(s: String) -> Self {
        Documentation(s)
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
            TSKind::preproc_define
            | TSKind::enum_entry
            | TSKind::variable_declaration
            | TSKind::struct_field
            | TSKind::dynamic_array_declaration => {
                if let Some(parent) = node.parent() {
                    if matches!(
                        TSKind::from(parent),
                        TSKind::global_variable_declaration
                            | TSKind::variable_declaration_statement
                    ) {
                        node = parent;
                    }
                }
                if let Some(prev_node) = node.prev_sibling() {
                    if TSKind::from(prev_node) == TSKind::preproc_pragma {
                        let text = prev_node.utf8_text(source).ok()?;
                        if text.starts_with("#pragma deprecated") {
                            pragma = Some(text.trim_start_matches("#pragma deprecated").trim());
                        }
                    }
                }
                while let Some(next_node) = node.next_sibling() {
                    if node.range().end_point.row != next_node.range().start_point.row {
                        break;
                    }
                    if TSKind::from(next_node) == TSKind::anon_COMMA {
                        node = next_node;
                        continue;
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
            TSKind::parameter_declaration => {
                let name = node.child_by_field_name("name")?.utf8_text(source).ok()?;
                let fn_node = node.parent()?.parent()?;
                let fn_doc = Documentation::from_node(fn_node, source)?;
                docs = fn_doc
                    .param_description(name)?
                    .0
                    .lines()
                    .map(|l| l.to_string())
                    .collect();
                docs.reverse();
            }

            TSKind::function_declaration
            | TSKind::function_definition
            | TSKind::typedef
            | TSKind::typeset
            | TSKind::functag
            | TSKind::funcenum
            | TSKind::r#struct
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
                        let text = prev_node.utf8_text(source).ok()?;
                        if text.starts_with("#pragma deprecated") {
                            pragma = Some(text.trim_start_matches("#pragma deprecated").trim());
                        }
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
            .trim()
            .to_string()
    }

    /// Extracts the description of a parameter from the documentation.
    pub fn param_description(&self, param_name: &str) -> Option<Self> {
        let re = Regex::new(&format!(r"\s*@param\s+{}\s+", regex::escape(param_name))).ok()?;
        let start = re.find(self.as_str())?.end();
        let end = self.as_str()[start..]
            .find('@')
            .map(|i| start + i)
            .unwrap_or_else(|| self.as_str().len());
        Some(self.as_str()[start..end].trim().to_string().into())
    }
}

fn comment_to_doc(text: &str) -> String {
    lazy_static! {
        static ref RE1: Regex = Regex::new(r"^\s*/(?:\*){2}<?\s*").unwrap();
        static ref RE2: Regex = Regex::new(r"\*/$").unwrap();
        static ref RE3: Regex = Regex::new(r"^\s*//\s*").unwrap();
        static ref RE4: Regex = Regex::new(r"\r?\n\s*\*\s*").unwrap();
        static ref RE5: Regex = Regex::new(r"^\s*\*\s*").unwrap();
    }
    let text = RE1.replace_all(text, "").into_owned();
    let text = RE2.replace_all(&text, "").into_owned();
    let text = RE3.replace_all(&text, "").into_owned();
    let text = RE4.replace_all(&text, "\n").into_owned();
    let text = RE5.replace_all(&text, "").into_owned();

    text
}
