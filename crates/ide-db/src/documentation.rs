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
            TSKind::function_declaration | TSKind::function_definition => {
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
                    docs.push(
                        prev_node
                            .utf8_text(source)
                            .ok()?
                            .trim_start_matches("//")
                            .trim_start_matches("/*")
                            .trim_start_matches("*/")
                            .trim_start_matches('*')
                            .trim(),
                    );
                    if prev_node.range().start_point.row != prev_node.range().end_point.row {
                        // Only keep one multi-line comment
                        break;
                    }
                    node = prev_node;
                }
            }
            _ => (),
        }
        docs.reverse();
        Documentation::new(docs.join("\n")).into()
    }
}
