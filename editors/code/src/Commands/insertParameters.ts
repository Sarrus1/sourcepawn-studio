import { window, commands, SignatureHelp } from "vscode";

export async function run(args: any): Promise<void> {
  const document = window.activeTextEditor.document;
  const position = window.activeTextEditor.selection.active;
  const linetext = document.lineAt(position).text;

  if (
    linetext[position.character] === ")" &&
    linetext[position.character - 1] === "("
  ) {
    const signatureHelp = (await commands.executeCommand(
      "vscode.executeSignatureHelpProvider",
      document.uri,
      position
    )) as SignatureHelp;

    const label = signatureHelp.signatures[0].label;
    const parameters = label.substring(
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
