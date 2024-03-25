use lazy_static::lazy_static;
use paths::AbsPathBuf;
use regex::Regex;

/// Severity levels of spcomp errors.
#[derive(Debug, Clone)]
pub enum SpCompSeverity {
    Warning,
    Error,
    FatalError,
}

impl SpCompSeverity {
    /// Convert to a [LSP DiagnosticSeverity](lsp_types::DiagnosticSeverity).
    pub fn to_lsp_severity(&self) -> lsp_types::DiagnosticSeverity {
        match self {
            SpCompSeverity::Warning => lsp_types::DiagnosticSeverity::WARNING,
            SpCompSeverity::Error => lsp_types::DiagnosticSeverity::ERROR,
            SpCompSeverity::FatalError => lsp_types::DiagnosticSeverity::ERROR,
        }
    }
}

/// Representation of an spcomp error.
#[derive(Debug, Clone)]
pub struct SpCompDiagnostic {
    /// [Path](AbsPathBuf) of the document where the error comes from.
    path: AbsPathBuf,

    /// Line index of the error.
    line_index: u32,

    /// Severity of the error.
    severity: SpCompSeverity,

    /// Code of the error.
    code: String,

    /// Message of the error.
    message: String,
}

impl SpCompDiagnostic {
    pub fn path(&self) -> &AbsPathBuf {
        &self.path
    }

    pub fn line_index(&self) -> u32 {
        self.line_index
    }

    pub fn severity(&self) -> &SpCompSeverity {
        &self.severity
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn try_from_line(line: &str) -> Option<Self> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"([:/\\A-Za-z\-_0-9. ]*)\((\d+)+\) : (?:(error|fatal error|warning) ([0-9]*)):\s+(.*)"
            )
            .expect("Failed to compile spcomp error regex.");
        }
        let capture = RE.captures(line)?;
        Some(Self {
            path: AbsPathBuf::try_from(capture.get(1)?.as_str()).ok()?,
            line_index: capture.get(2)?.as_str().parse::<u32>().ok()? - 1,
            severity: match capture.get(3)?.as_str() {
                "warning" => SpCompSeverity::Warning,
                "error" => SpCompSeverity::Error,
                "fatal error" => SpCompSeverity::FatalError,
                _ => unreachable!(),
            },
            code: capture.get(4)?.as_str().to_string(),
            message: capture.get(5)?.as_str().to_string(),
        })
    }
}

/// Return a [vector](Vec) of [strings](String) of the arguments to run spcomp.
pub fn build_args(
    root_path: &AbsPathBuf,
    out_path: &AbsPathBuf,
    includes_directories: &[AbsPathBuf],
    linter_arguments: &[String],
) -> Vec<String> {
    let mut args = vec![root_path.to_string()];
    args.extend(
        includes_directories
            .iter()
            .map(|includes_directory| format!("-i{}", includes_directory)),
    );
    if let Some(parent_path) = root_path.parent() {
        args.push(format!("-i{}", parent_path));
        let include_path = parent_path.join("include");
        if std::fs::metadata(&include_path).is_ok() {
            args.push(format!("-i{}", include_path));
        }
    }

    args.push(format!("-o{}", out_path));
    args.push("--syntax-only".to_string());

    args.extend_from_slice(linter_arguments);

    args
}
