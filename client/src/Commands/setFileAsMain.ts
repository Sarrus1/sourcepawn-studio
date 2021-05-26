import * as vscode from "vscode";

export async function run(args: any) {
  let uri: vscode.Uri = args.document.uri;
  if (typeof uri === "undefined") {
    vscode.window.showErrorMessage("No file are selected");
    return 1;
  }
  vscode.workspace
    .getConfiguration("sourcepawn")
    .update("MainPath", uri.fsPath);
  return 0;
}
