import { languages } from "vscode";

// Register and export the DiagnosticsCollection objects to be used by other modules.
export const compilerDiagnostics = languages.createDiagnosticCollection(
  "compiler"
);

export const cfgDiagnostics = languages.createDiagnosticCollection("cfg");
