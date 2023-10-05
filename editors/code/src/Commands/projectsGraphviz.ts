import * as vscode from "vscode";
import { projectsGraphviz, ProjectsGraphvizParams } from "../lsp_ext";
import { getCtxFromUri } from "../spIndex";

export async function run(args: any) {
  if (!vscode.extensions.getExtension("graphviz-interactive-preview.preview")) {
    vscode.window
      .showErrorMessage(
        "The extension 'graphviz-interactive-preview' is required to run this command.",
        "Install"
      )
      .then((msg) => {
        if (msg === "Install") {
          vscode.commands.executeCommand(
            "workbench.extensions.search",
            "graphviz-interactive-preview"
          );
        }
      });
    return;
  }
  const params: ProjectsGraphvizParams = {};
  const doc = vscode.window.activeTextEditor?.document;
  if (doc === undefined) {
    vscode.window.showErrorMessage("Open a document to use this command.");
    return;
  }
  const ctx = getCtxFromUri(doc.uri);
  params.textDocument =
    ctx?.client.code2ProtocolConverter.asTextDocumentIdentifier(doc);
  let content = await ctx?.client.sendRequest(projectsGraphviz, params);
  if (content === undefined) {
    content = "";
  }
  let options = {
    content,
    title: "Sourcepawn Dependency Graph",
  };

  vscode.commands.executeCommand(
    "graphviz-interactive-preview.preview.beside",
    options
  );
}
