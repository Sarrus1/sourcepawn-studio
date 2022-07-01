import {
  TextDocument,
  workspace as Workspace,
  Range,
  Diagnostic,
  DiagnosticSeverity,
} from "vscode";

import { parse, SyntaxError } from "../Parser/kvParser/kvParser";
import {
  KeyValue,
  ParserOutput,
  ParserRange,
} from "../Parser/kvParser/kvParserInterfaces";
import { kvDiagnostics } from "./Linter/compilerDiagnostics";

/**
 * Lint a Valve Key Value TextDocument object and add its diagnostics to the collection.
 * @param  {TextDocument} document    The document to lint.
 * @returns void
 */
export async function refreshKVDiagnostics(document: TextDocument) {
  await null;

  // Check if the setting to activate the linter is set to true.
  const workspaceFolder = Workspace.getWorkspaceFolder(document.uri);
  const enableLinter = Workspace.getConfiguration(
    "sourcepawn",
    workspaceFolder
  ).get<boolean>("enableLinter");

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
      const range = new Range(
        e.location.start.line - 1,
        e.location.start.column - 1,
        e.location.end.line - 1,
        e.location.end.column - 1
      );

      const msg = e.name + " " + e.message;
      const diag = new Diagnostic(range, msg);
      kvDiagnostics.set(document.uri, [diag]);
    }
    return;
  }
  kvDiagnostics.set(document.uri, lookForDuplicates(parsed.keyvalues));
}

function lookForDuplicates(keyvalues: KeyValue[]): Diagnostic[] {
  const map = new Map<string, Range[]>();
  let diagnostics: Diagnostic[] = [];
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
        new Diagnostic(
          prevRanges[0],
          "Duplicate object key",
          DiagnosticSeverity.Warning
        )
      );
    }
    prevRanges.push(range);
    diagnostics.push(
      new Diagnostic(range, "Duplicate object key", DiagnosticSeverity.Warning)
    );
  }
  return diagnostics;
}

function parserRangeToRange(range: ParserRange): Range {
  return new Range(
    range.start.line - 1,
    range.start.column - 1,
    range.end.line - 1,
    range.end.column - 1
  );
}
