import { workspace as Workspace, window, Uri } from "vscode";

export async function run(args: any) {
  let uri: Uri = args.document.uri;
  if (uri === undefined) {
    window.showErrorMessage("No file are selected");
    return 1;
  }
  let workspaceFolder =
    uri === undefined ? undefined : Workspace.getWorkspaceFolder(uri);
  Workspace.getConfiguration("sourcepawn", workspaceFolder).update(
    "MainPath",
    uri.fsPath
  );
  return 0;
}
