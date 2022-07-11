import {
  Diagnostic,
  DiagnosticCollection,
  Position,
  Range,
  DiagnosticSeverity,
} from "vscode";
import { URI } from "vscode-uri";

import { errorDetails } from "../../Misc/errorMessages";

function generateDetailedError(errorCode: string, errorMsg: string): string {
  if (errorDetails[errorCode] !== undefined) {
    errorMsg += "\n\n" + errorDetails[errorCode];
  }
  return errorMsg;
}
export function parseSPCompErrors(
  stdout: string,
  compilerDiagnostics: DiagnosticCollection
): void {
  const DocumentDiagnostics: Map<string, Diagnostic[]> = new Map();
  const re = /([:\/\\A-Za-z\-_0-9. ]*)\((\d+)+\) : ((error|fatal error|warning) ([0-9]*)):\s+(.*)/gm;
  let matches: RegExpExecArray | null;
  let diagnostics: Diagnostic[];
  do {
    matches = re.exec(stdout.toString() || "");
    if (matches) {
      const range = new Range(
        new Position(Number(matches[2]) - 1, 0),
        new Position(Number(matches[2]) - 1, 256)
      );
      const severity =
        matches[4] === "warning"
          ? DiagnosticSeverity.Warning
          : DiagnosticSeverity.Error;
      const uri = URI.file(matches[1]).toString();
      diagnostics = DocumentDiagnostics.get(uri) || [];

      const message = generateDetailedError(matches[5], matches[6]);
      const diagnostic = new Diagnostic(range, message, severity);
      diagnostics.push(diagnostic);
      DocumentDiagnostics.set(uri, diagnostics);
    }
  } while (matches);
  compilerDiagnostics.clear();
  for (const [uri, diagnostics] of DocumentDiagnostics) {
    compilerDiagnostics.set(URI.parse(uri), diagnostics);
  }
}
