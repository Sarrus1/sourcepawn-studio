import {
  workspace as Workspace,
  window,
  languages,
  ExtensionContext,
} from "vscode";

import { refreshKVDiagnostics } from "../kvLinter";
import { kvDiagnostics } from "./compilerDiagnostics";

export function registerKVLinter(context: ExtensionContext) {
  context.subscriptions.push(languages.createDiagnosticCollection("kv"));
  context.subscriptions.push(
    window.onDidChangeActiveTextEditor((editor) => {
      if (editor) {
        refreshKVDiagnostics(editor.document);
      }
    })
  );
  context.subscriptions.push(
    Workspace.onDidCloseTextDocument((document) => {
      kvDiagnostics.delete(document.uri);
    })
  );
}
