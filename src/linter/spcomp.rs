use std::{env, fs, path::PathBuf, process::Command};

use fxhash::FxHashMap;
use lazy_static::lazy_static;
use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, Url};
use regex::Regex;

use crate::store::Store;

/// Severity levels of spcomp errors.
#[derive(Debug)]
enum SPCompSeverity {
    Warning,
    Error,
    FatalError,
}

impl SPCompSeverity {
    /// Convert to a [LSP DiagnosticSeverity](lsp_types::DiagnosticSeverity).
    fn to_lsp_severity(&self) -> DiagnosticSeverity {
        match self {
            SPCompSeverity::Warning => DiagnosticSeverity::WARNING,
            SPCompSeverity::Error => DiagnosticSeverity::ERROR,
            SPCompSeverity::FatalError => DiagnosticSeverity::ERROR,
        }
    }
}

/// Representation of an spcomp error.
#[derive(Debug)]
pub(crate) struct SPCompDiagnostic {
    /// [Uri](Url) of the document where the error comes from.
    uri: Url,

    /// Line index of the error.
    line_index: u32,

    /// Severity of the error.
    severity: SPCompSeverity,

    /// Message of the error.
    message: String,
}

impl SPCompDiagnostic {
    /// Convert to an [LSP Diagnostic](lsp_types::Diagnostic).
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
    /// Run spcomp and extract the potential errors from its output.
    ///
    /// # Errors
    ///
    /// Will throw an error if spcomp fails to run (outside of errors related to the compilation).
    ///
    /// # Arguments
    ///
    /// * `uri` - [Uri](Url) of the file to compile.
    pub(crate) fn get_spcomp_diagnostics(
        &mut self,
        uri: Url,
    ) -> anyhow::Result<FxHashMap<Url, Vec<SPCompDiagnostic>>> {
        let output = Command::new(self.environment.options.spcomp_path.to_str().unwrap())
            .args(self.build_args(&uri))
            .output();
        let out_path = self.get_out_path();
        if out_path.exists() {
            let _ = fs::remove_file(out_path);
        }

        let output = output?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.is_empty() {
            return Err(anyhow::anyhow!(
                "Failed to run spcomp with error: {}",
                stderr
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut res: FxHashMap<Url, Vec<SPCompDiagnostic>> = FxHashMap::default();
        for diagnostic in parse_spcomp_errors(&stdout) {
            if let Some(diagnostics) = res.get_mut(&diagnostic.uri) {
                diagnostics.push(diagnostic);
            } else {
                res.insert(diagnostic.uri.clone(), vec![diagnostic]);
            }
        }

        Ok(res)
    }

    /// Return a [vector](Vec) of [strings](String) of the arguments to run spcomp.
    ///
    /// # Arguments
    ///
    /// * `uri` - [Uri](Url) of the file to compile.
    fn build_args(&mut self, uri: &Url) -> Vec<String> {
        let file_path = uri.to_file_path().unwrap();
        let mut args = vec![file_path.to_str().unwrap().to_string()];
        for includes_directory in self.environment.options.includes_directories.iter() {
            args.push(format!("-i{}", includes_directory.to_str().unwrap()));
        }
        let parent_path = file_path.parent().unwrap();
        args.push(format!("-i{}", parent_path.to_str().unwrap()));
        let include_path = parent_path.join("include");
        if include_path.exists() {
            args.push(format!("-i{}", include_path.to_str().unwrap()));
        }

        args.push(format!("-o{}", self.get_out_path().to_str().unwrap()));

        args.push("--syntax-only".to_string());

        args.extend(self.environment.options.linter_arguments.clone());

        args
    }

    /// Generate a temporary path for the output of spcomp. This is not needed with the `--syntax-only` switch.
    fn get_out_path(&self) -> PathBuf {
        env::temp_dir().join(format!("{}.smx", self.environment.sp_comp_uuid))
    }
}

/// Return a [vector](Vec) of [SPCompDiagnostics](SPCompDiagnostic) of the errors that spcomp threw.
///
/// # Arguments
///
/// * `stdout` - Standard output of spcomp.
fn parse_spcomp_errors(stdout: &str) -> Vec<SPCompDiagnostic> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"([:/\\A-Za-z\-_0-9. ]*)\((\d+)+\) : ((error|fatal error|warning) ([0-9]*)):\s+(.*)"
        )
        .unwrap();
    }
    let mut diagnostics = vec![];
    for captures in RE.captures_iter(stdout) {
        diagnostics.push(SPCompDiagnostic {
            uri: Url::from_file_path(captures.get(1).unwrap().as_str()).unwrap(),
            line_index: captures.get(2).unwrap().as_str().parse::<u32>().unwrap() - 1,
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
