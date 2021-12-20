import {
  DocumentFormattingEditProvider,
  TextDocument,
  FormattingOptions,
  CancellationToken,
  ProviderResult,
  TextEdit,
  workspace as Workspace,
  Position,
  Range,
  window,
} from "vscode";

export class CFGDocumentFormattingEditProvider
  implements DocumentFormattingEditProvider {
  public provideDocumentFormattingEdits(
    document: TextDocument,
    options: FormattingOptions,
    token: CancellationToken
  ): ProviderResult<TextEdit[]> {
    let workspaceFolder = Workspace.getWorkspaceFolder(document.uri);
    const result = [];

    // Get the user's settings.
    let insertSpaces: boolean = Workspace.getConfiguration(
      "editor",
      workspaceFolder
    ).get("insertSpaces");
    let tabSize: number = Workspace.getConfiguration(
      "editor",
      workspaceFolder
    ).get("tabSize");

    // Apply user settings
    const start = new Position(0, 0);
    const end = new Position(
      document.lineCount - 1,
      document.lineAt(document.lineCount - 1).text.length
    );
    let range = new Range(start, end);

    let text = formatCFGText(document.getText(), insertSpaces, tabSize);

    // If process failed,
    if (text === "") {
      window.showErrorMessage(
        "The formatter failed to run, check the console for more details."
      );
      return;
    }
    result.push(new TextEdit(range, text));
    return result;
  }
}

function formatCFGText(
  text: string,
  insertSpaces: boolean,
  tabSize: number
): string {
  let newText = "";
  let isSingleQuoteOpen = false;
  let isDoubleQuoteOpen = false;
  let bracketCounter = 0;
  let indentChar = insertSpaces ? " ".repeat(tabSize) : "\t".repeat(tabSize);
  let firstStringOfLineReached = false;

  for (let char of text) {
    if (char === "'" && !isDoubleQuoteOpen) {
      if (isSingleQuoteOpen && !firstStringOfLineReached) {
        newText += "'" + indentChar;
        firstStringOfLineReached = true;
      } else {
        newText += char;
      }
      isSingleQuoteOpen = !isSingleQuoteOpen;
    } else if (char === '"' && !isSingleQuoteOpen) {
      if (isDoubleQuoteOpen && !firstStringOfLineReached) {
        newText += '"' + indentChar;
        firstStringOfLineReached = true;
      } else {
        newText += char;
      }
      isDoubleQuoteOpen = !isDoubleQuoteOpen;
    } else if (char === "{" && !(isSingleQuoteOpen || isDoubleQuoteOpen)) {
      // Make sure to trim all previous spaces
      newText = newText.replace(/\s*$/, "");
      firstStringOfLineReached = false;
      newText += "\n" + indentChar.repeat(bracketCounter);
      bracketCounter++;
      newText += char;
      newText += "\n" + indentChar.repeat(bracketCounter);
    } else if (char === "}" && !(isSingleQuoteOpen || isDoubleQuoteOpen)) {
      bracketCounter--;

      // End of the file
      if (/\s/.test(newText.charAt(newText.length - 1))) {
        newText += char;
        // Remove previously added indent
        newText = newText.replace(/\s*}$/, "\n}");
        break;
      }
      newText += "\n" + indentChar.repeat(bracketCounter);
      newText += char;
      newText += "\n" + indentChar.repeat(bracketCounter);
    } else if (!/\s|\n/.test(char) || isSingleQuoteOpen || isDoubleQuoteOpen) {
      // Don't append existing spaces.
      newText += char;
    }
  }
  return newText;
}
