import * as vscode from "vscode";
import { clangFormat } from "./smFormatterPath";

export class DocumentFormattingEditProvider
  implements vscode.DocumentFormattingEditProvider {
  public provideDocumentFormattingEdits(
    document: vscode.TextDocument,
    options: vscode.FormattingOptions,
    token: vscode.CancellationToken
  ): vscode.ProviderResult<vscode.TextEdit[]> {
    const result = [];
		// Get the user's settings.
		let insert_spaces : boolean = vscode.workspace.getConfiguration("editor").get("insertSpaces");
		let UseTab : string = insert_spaces? "Never":"Always";
		let tabSize : string = vscode.workspace.getConfiguration("editor").get("tabSize");
		
		let default_styles : string[] = vscode.workspace.getConfiguration("sourcepawnLanguageServer").get("formatterSettings");
    
		let default_style: string = "{" + default_styles.join(", ") + "}";

		// Apply user settings
		default_style = default_style.replace(/\${TabSize}/, tabSize).replace(/\${UseTab}/, UseTab);
    const start = new vscode.Position(0, 0);
    const end = new vscode.Position(
      document.lineCount - 1,
      document.lineAt(document.lineCount - 1).text.length
    );
    const range = new vscode.Range(start, end);
    let text: string = clangFormat(document, "utf-8", default_style);

    // If process failed,
    if (text === "") {
      vscode.window.showErrorMessage(
        "The formatter failed to run, check the console for more details."
      );
      return;
    }
    // clang-format gets confused with 'public' so we have to replace it manually.
    text = text.replace(/^ *public\s*\n/gm, "public ");
    result.push(new vscode.TextEdit(range, text));
    return result;
  }

  Callback(e) {
    console.debug("hey");
    console.debug(e);
  }
}
