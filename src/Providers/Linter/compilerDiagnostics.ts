import { languages } from "vscode";

// Register and export the DiagnosticsCollection object to be used by other modules.
export const compilerDiagnostics = languages.createDiagnosticCollection(
  "compiler"
);
