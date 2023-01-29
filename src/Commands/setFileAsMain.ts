import { workspace as Workspace, window } from "vscode";
import { URI } from "vscode-uri";

/**
 * Callback for the Set Current File As Main command.
 * @param  {URI} args URI of the document to be set as main.
 * @returns Promise
 */
export async function run(args: URI): Promise<number> {
  if (args === undefined) {
    if (window.activeTextEditor === undefined) {
      return 1;
    }
    args = window.activeTextEditor.document.uri;
  }
  const workspaceFolder = Workspace.getWorkspaceFolder(args);
  await Workspace.getConfiguration(
    "SourcePawnLanguageServer",
    workspaceFolder
  ).update("MainPath", args.fsPath);
  return 0;
}
