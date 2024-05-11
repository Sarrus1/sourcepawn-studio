import * as vscode from "vscode";

import { KvErrorKind, lintKeyvalue, Range } from "valve_kv_tools";

// Register and export the DiagnosticsCollection objects to be used by other modules.
export const kvDiagnostics = vscode.languages.createDiagnosticCollection("kv");

export function registerKVLinter(context: vscode.ExtensionContext) {
  context.subscriptions.push(kvDiagnostics);
  context.subscriptions.push(
    vscode.window.onDidChangeActiveTextEditor((editor) => {
      if (editor) {
        refreshKVDiagnostics(editor.document);
      }
    })
  );
  context.subscriptions.push(
    vscode.workspace.onDidChangeTextDocument((event) => {
      refreshKVDiagnostics(event.document);
    })
  );
  context.subscriptions.push(
    vscode.workspace.onDidCloseTextDocument((document) => {
      kvDiagnostics.delete(document.uri);
    })
  );
}

/**
 * Lint a Valve Key Value TextDocument object and add its diagnostics to the collection.
 * @param  {TextDocument} document    The document to lint.
 * @returns void
 */
export async function refreshKVDiagnostics(document: vscode.TextDocument) {
  // Check if the setting to activate the linter is set to true.
  const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
  const enableLinter = vscode.workspace
    .getConfiguration("amxxpawn", workspaceFolder)
    .get<boolean>("enableLinter");

  // Stop early if linter is disabled.
  if (!enableLinter || document.languageId !== "valve-kv") {
    kvDiagnostics.set(document.uri, []);
    return;
  }
  kvDiagnostics.delete(document.uri);

  const diagnostics: vscode.Diagnostic[] = [];
  lintKeyvalue(document.getText()).forEach((e) => {
    let severity: vscode.DiagnosticSeverity;
    switch (e.kind) {
      case KvErrorKind.SyntaxError:
        severity = vscode.DiagnosticSeverity.Error;
        break;
      case KvErrorKind.DuplicateError:
        severity = vscode.DiagnosticSeverity.Hint;
    }
    diagnostics.push({
      range: rangeToVscodeRange(e.range),
      message: e.message,
      severity,
    });
    if (e.kind === KvErrorKind.DuplicateError) {
      e.additionalRanges.forEach((range: Range) =>
        diagnostics.push({
          range: rangeToVscodeRange(range),
          message: e.message,
          severity: vscode.DiagnosticSeverity.Warning,
        })
      );
    }
  });
  kvDiagnostics.set(document.uri, diagnostics);
}

function rangeToVscodeRange(range: Range): vscode.Range {
  return new vscode.Range(
    range.start.line,
    range.start.character,
    range.end.line,
    range.end.character
  );
}
