use base_db::Tree;
use fxhash::FxHashSet;
use hir::{AnyDiagnostic, Semantics};
use hir_def::{InFile, NodePtr};
use ide_db::RootDatabase;
use line_index::{TextRange, TextSize};
use queries::ERROR_QUERY;
use streaming_iterator::StreamingIterator;
use syntax::utils::ts_range_to_text_range;
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
    pub u_range: TextRange,
    pub severity: Severity,
    pub unused: bool,
    pub experimental: bool,
    // pub fixes: Option<Vec<Assist>>,
    // The node that will be affected by `#[allow]` and similar attributes.
}

impl Diagnostic {
    fn new_with_syntax_node_ptr(
        ctx: &DiagnosticsContext<'_>,
        code: DiagnosticCode,
        message: impl Into<String>,
        node: InFile<NodePtr>,
    ) -> Diagnostic {
        let s_range = TextRange::new(
            TextSize::new(node.value.start_byte() as u32),
            TextSize::new(node.value.end_byte() as u32),
        );

        Self::new_for_s_range(ctx, code, message, s_range)
    }

    fn new_for_s_range(
        ctx: &DiagnosticsContext<'_>,
        code: DiagnosticCode,
        message: impl Into<String>,
        s_range: TextRange,
    ) -> Self {
        let preprocessing_results = ctx.sema.preprocess_file(ctx.file_id);

        Diagnostic::new_for_u_range(
            code,
            message,
            preprocessing_results
                .source_map()
                .closest_u_range_always(s_range),
        )
    }

    fn new_for_u_range(
        code: DiagnosticCode,
        message: impl Into<String>,
        u_range: TextRange,
    ) -> Self {
        Diagnostic {
            code,
            message: message.into(),
            u_range,
            severity: match code {
                DiagnosticCode::SpCompError(_) => Severity::Error,
                DiagnosticCode::SpCompWarning(_) => Severity::Warning,
                DiagnosticCode::Lint(_, s) => s,
            },
            unused: false,
            experimental: false,
        }
    }

    #[allow(unused)]
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
    #[allow(unused)]
    config: &'a DiagnosticsConfig,
    sema: Semantics<'a, RootDatabase>,
    file_id: FileId,
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
    let tree = sema.parse(file_id);
    let source = sema.preprocessed_text(file_id);
    let mut res = Vec::new();

    let file = sema.file_to_def(file_id);
    let ctx = DiagnosticsContext {
        config,
        sema,
        file_id,
    };

    syntax_error_diagnostics(&ctx, &source, &tree, &mut res);

    let mut diags = Vec::new();
    file.diagnostics(db, &mut diags);
    for diag in diags {
        let d = match diag {
            AnyDiagnostic::UnresolvedField(d) => handlers::unresolved_field::f(&ctx, &d),
            AnyDiagnostic::UnresolvedMethodCall(d) => handlers::unresolved_method_call::f(&ctx, &d),
            AnyDiagnostic::UnresolvedInclude(d) => handlers::unresolved_include::f(&ctx, &d),
            AnyDiagnostic::UnresolvedConstructor(d) => {
                handlers::unresolved_constructor::f(&ctx, &d)
            }
            AnyDiagnostic::UnresolvedNamedArg(d) => handlers::unresolved_named_arg::f(&ctx, &d),
            AnyDiagnostic::IncorrectNumberOfArguments(d) => {
                handlers::incorrect_number_of_arguments::f(&ctx, &d)
            }
            AnyDiagnostic::UnresolvedInherit(d) => handlers::unresolved_inherit::f(&ctx, &d),
            AnyDiagnostic::PreprocessorEvaluationError(d) => {
                handlers::preprocessor_evaluation_error::f(&ctx, &d)
            }
            AnyDiagnostic::UnresolvedMacro(d) => handlers::unresolved_macro::f(&ctx, &d),
            AnyDiagnostic::InactiveCode(d) => handlers::inactive_code::f(&ctx, &d),
            AnyDiagnostic::InvalidUseOfThis(d) => handlers::invalid_use_of_this::f(&ctx, &d),
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
/// * `ctx` - [DiagnosticsContext](DiagnosticsContext) of the document.
/// * `source` - Preprocessed text of the document.
/// * `tree` - [Tree](base_db::Tree) of the document.
/// * `diagnostics` - [Vec](std::vec::Vec) of [Diagnostic](crate::Diagnostic) to add the
///   syntax errors to.
fn syntax_error_diagnostics(
    ctx: &DiagnosticsContext,
    source: &str,
    tree: &Tree,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.captures(&ERROR_QUERY, tree.root_node(), source.as_bytes());
    while let Some((match_, _)) = matches.next() {
        diagnostics.extend(match_.captures.iter().map(|c| {
            ts_error_to_diagnostic(ctx, c.node).unwrap_or_else(|| {
                Diagnostic::new_for_s_range(
                    ctx,
                    DiagnosticCode::SpCompError("syntax-error"),
                    c.node.to_sexp(),
                    ts_range_to_text_range(&c.node.range()),
                )
            })
        }));
    }

    missing_nodes(ctx, tree.root_node(), diagnostics);
}

/// Capture all the missing nodes of a document and add them to its Local Diagnostics.
///
/// # Arguments
///
/// * `ctx` - [DiagnosticsContext](DiagnosticsContext) of the document.
/// * `node` - [Node](tree_sitter::Node) to scan.
/// * `diagnostics` - [Vec](std::vec::Vec) of [Diagnostic](crate::Diagnostic) to add the missing nodes to.
fn missing_nodes(
    ctx: &DiagnosticsContext,
    node: tree_sitter::Node,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if node.is_missing() {
        let diagnostic = Diagnostic::new_for_s_range(
            ctx,
            DiagnosticCode::SpCompError("missing-node"),
            format!("expected `{}`", node.kind()),
            ts_range_to_text_range(&node.range()),
        );
        diagnostics.push(diagnostic);
    }

    for child in node.children(&mut node.walk()) {
        missing_nodes(ctx, child, diagnostics);
    }
}

/// Convert a tree-sitter error node to a diagnostic by using a [`LookaheadIterator`](tree_sitter::LookaheadIterator)
/// to get the expected nodes.
fn ts_error_to_diagnostic(ctx: &DiagnosticsContext, node: tree_sitter::Node) -> Option<Diagnostic> {
    let language = tree_sitter_sourcepawn::language();
    let first_lead_node = node.child(0)?;
    let mut lookahead = language.lookahead_iterator(first_lead_node.parse_state())?;
    let expected: Vec<_> = lookahead
        .iter_names()
        .map(|it| format!("`{}`", it))
        .collect();
    Diagnostic::new_for_s_range(
        ctx,
        DiagnosticCode::SpCompError("syntax-error"),
        format!("expected {:?}", expected.join(", ")),
        ts_range_to_text_range(&node.range()),
    )
    .into()
}
