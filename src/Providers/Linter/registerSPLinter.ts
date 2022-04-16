import {
  workspace as Workspace,
  window,
  languages,
  ExtensionContext,
} from "vscode";
import { refreshCfgDiagnostics } from "../cfgLinter";

import { refreshDiagnostics } from "../spLinter";
import { compilerDiagnostics } from "./compilerDiagnostics";
import { throttles } from "./throttles";

export function registerSPLinter(context: ExtensionContext) {
  context.subscriptions.push(compilerDiagnostics);
  context.subscriptions.push(
    window.onDidChangeActiveTextEditor((editor) => {
      if (editor) {
        refreshDiagnostics(editor.document);
        refreshCfgDiagnostics(editor.document);
      }
    })
  );
  context.subscriptions.push(
    Workspace.onDidCloseTextDocument((document) => {
      compilerDiagnostics.delete(document.uri);
      delete throttles[document.uri.path];
    })
  );
}
