use anyhow::{anyhow, Context};
use fxhash::FxHashMap;
use lazy_static::lazy_static;
use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, Url};
use regex::{Captures, Regex};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};
use uuid::Uuid;

lazy_static! {
    static ref SPCOMP_UUID: String = Uuid::new_v4().to_string();
}

/// Severity levels of spcomp errors.
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct SPCompDiagnostic {
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
    fn from_spcomp_captures(captures: Captures) -> Option<Self> {
        Some(Self {
            uri: Url::from_file_path(captures.get(1)?.as_str()).ok()?,
            line_index: captures.get(2)?.as_str().parse::<u32>().ok()? - 1,
            severity: match captures.get(4)?.as_str() {
                "warning" => SPCompSeverity::Warning,
                "error" => SPCompSeverity::Error,
                "fatal error" => SPCompSeverity::FatalError,
                _ => todo!(),
            },
            message: captures.get(6)?.as_str().to_string(),
        })
    }

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

/// Run spcomp and extract the potential errors from its output.
///
/// # Errors
///
/// Will throw an error if spcomp fails to run (outside of errors related to the compilation).
///
/// # Arguments
///
/// * `uri` - [Uri](Url) of the file to compile.
pub fn get_spcomp_diagnostics(
    uri: Url,
    spcomp_path: &Path,
    includes_directories: &[PathBuf],
    linter_arguments: &[String],
) -> anyhow::Result<FxHashMap<Url, Vec<SPCompDiagnostic>>> {
    let output = Command::new(
        spcomp_path
            .to_str()
            .context("Failed to convert spcomp path to string.")?,
    )
    .args(build_args(&uri, includes_directories, linter_arguments)?)
    .output();
    let out_path = get_out_path();
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
fn build_args(
    uri: &Url,
    includes_directories: &[PathBuf],
    linter_arguments: &[String],
) -> anyhow::Result<Vec<String>> {
    let file_path = uri
        .to_file_path()
        .map_err(|_| anyhow::anyhow!("Failed to convert uri to file path: {}", uri.to_string()))?;
    let mut args = vec![file_path
        .to_str()
        .ok_or_else(|| anyhow!("Failed to get file extension."))?
        .to_string()];
    args.extend(includes_directories.iter().flat_map(|includes_directory| {
        includes_directory
            .to_str()
            .map(|includes_directory| format!("-i{}", includes_directory))
    }));
    if let Some(parent_path) = file_path.parent() {
        if let Some(parent_path_str) = parent_path.to_str() {
            args.push(format!("-i{}", parent_path_str));
        }
        let include_path = parent_path.join("include");
        if include_path.exists() {
            if let Some(include_path_str) = include_path.to_str() {
                args.push(format!("-i{}", include_path_str));
            }
        }
    }

    if let Some(out_path_str) = get_out_path().to_str() {
        args.push(format!("-o{}", out_path_str));
    }
    args.push("--syntax-only".to_string());

    args.extend_from_slice(linter_arguments);

    Ok(args)
}

/// Generate a temporary path for the output of spcomp. This is not needed with the `--syntax-only` switch.
fn get_out_path() -> PathBuf {
    let mut path = SPCOMP_UUID.to_string();
    path.push_str(".smx");
    env::temp_dir().join(path)
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
        .expect("Failed to compile spcomp error regex.");
    }
    RE.captures_iter(stdout)
        .flat_map(SPCompDiagnostic::from_spcomp_captures)
        .collect()
}
