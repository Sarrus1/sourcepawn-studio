import * as vscode from "vscode";

import { parse, SyntaxError } from "./kvParser";
import { KeyValue, ParserOutput, ParserRange } from "./kvParserInterfaces";

// Register and export the DiagnosticsCollection objects to be used by other modules.
export const kvDiagnostics = vscode.languages.createDiagnosticCollection("kv");

export function registerKVLinter(context: vscode.ExtensionContext) {
  context.subscriptions.push(vscode.languages.createDiagnosticCollection("kv"));
  context.subscriptions.push(
    vscode.window.onDidChangeActiveTextEditor((editor) => {
      if (editor) {
        refreshKVDiagnostics(editor.document);
      }
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
  await null;

  // Check if the setting to activate the linter is set to true.
  const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
  const enableLinter = vscode.workspace
    .getConfiguration("sourcepawn", workspaceFolder)
    .get<boolean>("enableLinter");

  // Stop early if linter is disabled.
  if (!enableLinter || document.languageId !== "valve-kv") {
    kvDiagnostics.set(document.uri, []);
    return;
  }
  kvDiagnostics.delete(document.uri);

  let parsed: ParserOutput;
  try {
    parsed = parse(document.getText());
  } catch (e) {
    if (e instanceof SyntaxError) {
      const range = new vscode.Range(
        e.location.start.line - 1,
        e.location.start.column - 1,
        e.location.end.line - 1,
        e.location.end.column - 1
      );

      const msg = e.name + " " + e.message;
      const diag = new vscode.Diagnostic(range, msg);
      kvDiagnostics.set(document.uri, [diag]);
    }
    return;
  }
  kvDiagnostics.set(document.uri, lookForDuplicates(parsed.keyvalues));
}

function lookForDuplicates(keyvalues: KeyValue[]): vscode.Diagnostic[] {
  const map = new Map<string, vscode.Range[]>();
  let diagnostics: vscode.Diagnostic[] = [];
  for (let keyvalue of keyvalues) {
    const range = parserRangeToRange(keyvalue.key.loc);
    const key = keyvalue.key.txt;
    if (keyvalue.value.type === "section") {
      diagnostics = diagnostics.concat(
        lookForDuplicates(keyvalue.value.keyvalues)
      );
    }
    const prevRanges = map.get(key);
    if (prevRanges === undefined) {
      map.set(key, [range]);
      continue;
    }
    if (prevRanges.length === 1) {
      // Add the first diagnostic.
      diagnostics.push(
        new vscode.Diagnostic(
          prevRanges[0],
          "Duplicate object key",
          vscode.DiagnosticSeverity.Warning
        )
      );
    }
    prevRanges.push(range);
    diagnostics.push(
      new vscode.Diagnostic(
        range,
        "Duplicate object key",
        vscode.DiagnosticSeverity.Warning
      )
    );
  }
  return diagnostics;
}

function parserRangeToRange(range: ParserRange): vscode.Range {
  return new vscode.Range(
    range.start.line - 1,
    range.start.column - 1,
    range.end.line - 1,
    range.end.column - 1
  );
}
