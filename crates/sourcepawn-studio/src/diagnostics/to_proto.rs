use flycheck::SpCompDiagnostic;
use lsp_types::{Position, Range};

pub fn map_spcomp_diagnostic_to_lsp(diagnostic: &SpCompDiagnostic) -> lsp_types::Diagnostic {
    let range = Range {
        start: Position {
            line: diagnostic.line_index(),
            character: 0,
        },
        end: Position {
            line: diagnostic.line_index(),
            character: 1000,
        },
    };
    lsp_types::Diagnostic {
        range,
        severity: diagnostic.severity().to_lsp_severity().into(),
        code: Some(lsp_types::NumberOrString::String(
            diagnostic.code().to_string(),
        )),
        source: Some("spcomp".to_string()),
        message: diagnostic.message().to_string(),
        ..Default::default()
    }
}
