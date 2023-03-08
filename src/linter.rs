use std::process::Command;

use fxhash::FxHashMap;
use lazy_static::lazy_static;
use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, Url};
use regex::Regex;

use crate::store::Store;

#[derive(Debug)]
enum SPCompSeverity {
    Warning,
    Error,
    FatalError,
}

impl SPCompSeverity {
    fn to_lsp_severity(&self) -> DiagnosticSeverity {
        match self {
            SPCompSeverity::Warning => DiagnosticSeverity::WARNING,
            SPCompSeverity::Error => DiagnosticSeverity::ERROR,
            SPCompSeverity::FatalError => DiagnosticSeverity::ERROR,
        }
    }
}

#[derive(Debug)]
pub(crate) struct SPCompDiagnostic {
    uri: Url,
    line_index: u32,
    severity: SPCompSeverity,
    message: String,
}

impl SPCompDiagnostic {
    pub(crate) fn to_lsp_diagnostic(&self) -> Diagnostic {
        Diagnostic {
            range: Range {
                start: Position {
                    line: self.line_index,
                    character: 0,
                },
                end: Position {
                    line: self.line_index,
                    character: 1000,
                },
            },
            message: self.message.clone(),
            severity: Some(self.severity.to_lsp_severity()),
            ..Default::default()
        }
    }
}

impl Store {
    pub(crate) fn get_spcomp_diagnostics(
        &mut self,
        uri: Url,
    ) -> FxHashMap<Url, Vec<SPCompDiagnostic>> {
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .arg("/C")
                .args(self.build_args(uri))
                .output()
                .expect("failed to execute process")
        } else {
            Command::new("sh")
                .arg("-c")
                .args(self.build_args(uri))
                .output()
                .expect("failed to execute process")
        };
        self.clear_all_diagnostics();
        let output = String::from_utf8_lossy(&output.stdout);
        let mut res: FxHashMap<Url, Vec<SPCompDiagnostic>> = FxHashMap::default();
        for diagnostic in parse_spcomp_errors(&output) {
            if let Some(diagnostics) = res.get_mut(&diagnostic.uri) {
                diagnostics.push(diagnostic);
            } else {
                res.insert(diagnostic.uri.clone(), vec![diagnostic]);
            }
        }

        res
    }

    fn clear_all_diagnostics(&mut self) {
        for document in self.documents.values_mut() {
            document.diagnostics.clear();
        }
    }

    fn build_args(&mut self, uri: Url) -> Vec<String> {
        vec![
            self.environment
                .options
                .spcomp_path
                .to_str()
                .unwrap()
                .to_string(),
            uri.to_file_path().unwrap().to_str().unwrap().to_string(),
        ]
    }
}

fn parse_spcomp_errors(output: &str) -> Vec<SPCompDiagnostic> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"([:/\\A-Za-z\-_0-9. ]*)\((\d+)+\) : ((error|fatal error|warning) ([0-9]*)):\s+(.*)"
        )
        .unwrap();
    }
    let mut diagnostics = vec![];
    for captures in RE.captures_iter(output) {
        diagnostics.push(SPCompDiagnostic {
            uri: Url::from_file_path(captures.get(1).unwrap().as_str()).unwrap(),
            line_index: captures.get(2).unwrap().as_str().parse::<u32>().unwrap(),
            severity: match captures.get(4).unwrap().as_str() {
                "warning" => SPCompSeverity::Warning,
                "error" => SPCompSeverity::Error,
                "fatal error" => SPCompSeverity::FatalError,
                _ => todo!(),
            },
            message: captures.get(6).unwrap().as_str().to_string(),
        });
    }

    diagnostics
}
