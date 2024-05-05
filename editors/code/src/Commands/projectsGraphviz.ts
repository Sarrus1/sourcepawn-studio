import * as vscode from "vscode";
import { projectsGraphviz, ProjectsGraphvizParams } from "../lsp_ext";
import { getCtxFromUri, lastActiveEditor } from "../spIndex";

export async function run(args: any): Promise<void> {
  const params: ProjectsGraphvizParams = {};
  const doc = lastActiveEditor.document;
  if (!doc) {
    vscode.window.showErrorMessage("Open a document to use this command.");
    return;
  }

  const ctx = getCtxFromUri(doc.uri);
  params.textDocument =
    ctx?.client.code2ProtocolConverter.asTextDocumentIdentifier(doc);
  let content = await ctx?.client.sendRequest(projectsGraphviz, params) || "";
  let options = {
    content,
    title: "SourcePawn Dependency Graph",
  };

  vscode.commands
    .executeCommand("graphviz-interactive-preview.preview.beside", options)
    .then(
      (result) => {
        return;
      },
      (error) => {
        console.error(error);
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
      }
    );
}
