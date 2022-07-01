import { languages } from "vscode";

// Register and export the DiagnosticsCollection objects to be used by other modules.
export const compilerDiagnostics = languages.createDiagnosticCollection(
  "compiler"
);

export const parserDiagnostics = languages.createDiagnosticCollection("parser");

export const preDiagnostics = languages.createDiagnosticCollection(
  "preprocessor"
);

export const kvDiagnostics = languages.createDiagnosticCollection("kv");
