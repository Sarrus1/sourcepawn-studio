import { window, commands, SignatureHelp } from "vscode";

export async function run(args: any) {
  let document = window.activeTextEditor.document;
  let position = window.activeTextEditor.selection.active;
  const linetext = document.lineAt(position).text;

  if (
    linetext[position.character] === ")" &&
    linetext[position.character - 1] === "("
  ) {
    let signatureHelp = (await commands.executeCommand(
      "executeSignatureHelpProvider",
      document.uri,
      position
    )) as SignatureHelp;

    let label = signatureHelp.signatures[0].label;
    let parameters = label.substring(
      label.indexOf("(") + 1,
      label.indexOf(")")
    );

    window.activeTextEditor.edit(function (editBuilder) {
      editBuilder.insert(position, parameters);
    });
  } else {
    window.activeTextEditor.edit(function (editBuilder) {
      editBuilder.insert(position, "\t");
    });
  }
}
