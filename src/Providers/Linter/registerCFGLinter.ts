import {
  workspace as Workspace,
  window,
  languages,
  ExtensionContext,
} from "vscode";

import { refreshCfgDiagnostics } from "../cfgLinter";
import { cfgDiagnostics } from "./compilerDiagnostics";

export function registerCFGLinter(context: ExtensionContext) {
  context.subscriptions.push(languages.createDiagnosticCollection("cfg"));
  context.subscriptions.push(
    window.onDidChangeActiveTextEditor((editor) => {
      if (editor) {
        refreshCfgDiagnostics(editor.document);
      }
    })
  );
  context.subscriptions.push(
    Workspace.onDidCloseTextDocument((document) => {
      cfgDiagnostics.delete(document.uri);
    })
  );
}
