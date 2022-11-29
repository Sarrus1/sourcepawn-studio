use lazy_static::lazy_static;
use lsp_types::MarkupContent;
use regex::Regex;

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
