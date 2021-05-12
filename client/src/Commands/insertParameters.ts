import * as vscode from "vscode";

export async function run(args: any) {
  let document = vscode.window.activeTextEditor.document;
  let position = vscode.window.activeTextEditor.selection.active;
  const linetext = document.lineAt(position).text;

  if (
    linetext[position.character] === ")" &&
    linetext[position.character - 1] === "("
  ) {
    let signatureHelp = (await vscode.commands.executeCommand(
      "vscode.executeSignatureHelpProvider",
      document.uri,
      position
    )) as vscode.SignatureHelp;

    let label = signatureHelp.signatures[0].label;
    let parameters = label.substring(
      label.indexOf("(") + 1,
      label.indexOf(")")
    );

    vscode.window.activeTextEditor.edit(function (editBuilder) {
      editBuilder.insert(position, parameters);
    });
  }
}
