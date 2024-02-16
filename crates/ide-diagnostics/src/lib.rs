use base_db::{SourceDatabase, SourceDatabaseExt, Tree};
use fxhash::FxHashSet;
use hir::{AnyDiagnostic, Semantics};
use hir_def::{InFile, NodePtr};
use ide_db::RootDatabase;
use queries::ERROR_QUERY;
use syntax::utils::ts_range_to_lsp_range;
use tree_sitter::QueryCursor;
use vfs::FileId;

mod handlers;
mod queries;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DiagnosticCode {
    SpCompError(&'static str),
    SpCompWarning(&'static str),
    Lint(&'static str, Severity),
}

impl DiagnosticCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagnosticCode::SpCompError(r)
            | DiagnosticCode::SpCompWarning(r)
            | DiagnosticCode::Lint(r, _) => r,
        }
    }
}

#[derive(Debug)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub message: String,
    pub range: lsp_types::Range,
    pub severity: Severity,
    pub unused: bool,
    pub experimental: bool,
    // pub fixes: Option<Vec<Assist>>,
    // The node that will be affected by `#[allow]` and similar attributes.
}

impl Diagnostic {
    fn new(
        code: DiagnosticCode,
        message: impl Into<String>,
        range: lsp_types::Range,
    ) -> Diagnostic {
        let message = message.into();
        Diagnostic {
            code,
            message,
            range,
            severity: match code {
                DiagnosticCode::SpCompError(_) => Severity::Error,
                DiagnosticCode::SpCompWarning(_) => Severity::Warning,
                DiagnosticCode::Lint(_, s) => s,
            },
            unused: false,
            experimental: false,
        }
    }

    fn new_with_syntax_node_ptr(
        ctx: &DiagnosticsContext<'_>,
        code: DiagnosticCode,
        message: impl Into<String>,
        node: InFile<NodePtr>,
    ) -> Diagnostic {
        let file_id = node.file_id;
        let tree = ctx.sema.db.parse(file_id);
        let range = node.map(|x| x.to_node(&tree).range()).value;
        Diagnostic::new(code, message, ts_range_to_lsp_range(&range))
    }

    fn experimental(mut self) -> Diagnostic {
        self.experimental = true;
        self
    }

    // fn with_fixes(mut self, fixes: Option<Vec<Assist>>) -> Diagnostic {
    //     self.fixes = fixes;
    //     self
    // }

    fn with_unused(mut self, unused: bool) -> Diagnostic {
        self.unused = unused;
        self
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    WeakWarning,
}

struct DiagnosticsContext<'a> {
    config: &'a DiagnosticsConfig,
    sema: Semantics<'a, RootDatabase>,
}

pub struct DiagnosticsConfig {
    /// Whether native diagnostics are enabled.
    pub enabled: bool,
    pub disable_experimental: bool,
    pub disabled: FxHashSet<String>,
}

pub fn diagnostics(
    db: &RootDatabase,
    config: &DiagnosticsConfig,
    file_id: FileId,
) -> Vec<Diagnostic> {
    let sema = Semantics::new(db);
    let tree = db.parse(file_id);
    let source = db.file_text(file_id);
    let mut res = Vec::new();

    res.extend(syntax_error_diagnostics(&source, &tree));

    let file = sema.file_to_def(file_id);
    let ctx = DiagnosticsContext { config, sema };

    let mut diags = Vec::new();
    file.diagnostics(db, &mut diags);
    for diag in diags {
        let d = match diag {
            AnyDiagnostic::UnresolvedField(d) => handlers::unresolved_field::f(&ctx, &d),
            AnyDiagnostic::UnresolvedMethodCall(d) => handlers::unresolved_method_call::f(&ctx, &d),
            AnyDiagnostic::UnresolvedInclude(d) => handlers::unresolved_include::f(&ctx, &d),
            AnyDiagnostic::PreprocessorEvaluationError(d) => {
                handlers::preprocessor_evaluation_error::f(&ctx, &d)
            }
            AnyDiagnostic::InactiveCode(d) => handlers::inactive_code::f(&ctx, &d),
        };
        res.push(d);
    }

    res
}

/// Capture all the syntax errors of a document and add them to its Local Diagnostics.
/// Overrides all previous Local Diagnostics.
///
/// # Arguments
///
/// * `root_node` - [Root Node](tree_sitter::Node) of the document to scan.
/// * `disable_syntax_linter` - Whether or not the syntax linter should run.
pub fn syntax_error_diagnostics(source: &str, tree: &Tree) -> Vec<Diagnostic> {
    let mut res = Vec::new();
    let mut cursor = QueryCursor::new();
    let matches = cursor.captures(&ERROR_QUERY, tree.root_node(), source.as_bytes());
    for (match_, _) in matches {
        res.extend(match_.captures.iter().map(|c| {
            Diagnostic::new(
                DiagnosticCode::SpCompError("syntax-error"),
                c.node.to_sexp(),
                ts_range_to_lsp_range(&c.node.range()),
            )
        }));
    }
    // TODO: Add MISSING query here once https://github.com/tree-sitter/tree-sitter/issues/606 is fixed.

    res
}
