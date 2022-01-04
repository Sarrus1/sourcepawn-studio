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
/**
 * Formats a string parsed from a Sourcemod .kv file
 * @param  {string} text              The string to format
 * @param  {boolean} insertSpaces     Use tabs or spaces for indentation
 * @param  {number} tabSize           Tabsize of each indent
 * @returns string                    The formatted string
 */
function formatCFGText(
  text: string,
  insertSpaces: boolean,
  tabSize: number
): string {
  let newText = "";
  let isSingleQuoteOpen = false;
  let isDoubleQuoteOpen = false;
  let slashCounter = 0;
  let bracketCounter = 0;
  let indentChar = insertSpaces ? " ".repeat(tabSize) : "\t".repeat(tabSize);
  let firstStringOfLineReached = false;

  for (let char of text) {
    if (char === "'" && !isDoubleQuoteOpen && slashCounter < 2) {
      if (isSingleQuoteOpen && !firstStringOfLineReached) {
        newText += "'" + indentChar;
        firstStringOfLineReached = true;
      } else {
        newText += char;
      }
      isSingleQuoteOpen = !isSingleQuoteOpen;
    } else if (char === '"' && !isSingleQuoteOpen && slashCounter < 2) {
      if (isDoubleQuoteOpen && !firstStringOfLineReached) {
        newText += '"' + indentChar;
        firstStringOfLineReached = true;
      } else {
        newText += char;
      }
      isDoubleQuoteOpen = !isDoubleQuoteOpen;
    } else if (
      char === "{" &&
      !(isSingleQuoteOpen || isDoubleQuoteOpen) &&
      slashCounter < 2
    ) {
      // Make sure to trim all previous spaces
      newText = newText.replace(/\s*$/, "");
      firstStringOfLineReached = false;
      newText += "\n" + indentChar.repeat(bracketCounter);
      bracketCounter++;
      newText += char;
      newText += "\n" + indentChar.repeat(bracketCounter);
    } else if (
      char === "}" &&
      !(isSingleQuoteOpen || isDoubleQuoteOpen) &&
      slashCounter < 2
    ) {
      firstStringOfLineReached = false;
      bracketCounter--;
      newText += "\n" + indentChar.repeat(bracketCounter);
      newText += char;
      newText += "\n" + indentChar.repeat(bracketCounter);
    } else if (
      char === "/" &&
      slashCounter < 2 &&
      !(isSingleQuoteOpen || isDoubleQuoteOpen)
    ) {
      // Deal with comments
      slashCounter++;
      newText += char;
      if (slashCounter === 2) {
        newText += " ";
      } else if (slashCounter === 1 && firstStringOfLineReached) {
        newText =
          newText.slice(0, newText.length - 1) +
          " " +
          newText.slice(newText.length - 1, newText.length);
      }
    } else if (char === "\n" && slashCounter == 2) {
      slashCounter = 0;
      if (!firstStringOfLineReached) {
        newText += "\n" + indentChar.repeat(bracketCounter);
      }
    } else if (
      !/\s|\n/.test(char) ||
      isSingleQuoteOpen ||
      (isDoubleQuoteOpen && slashCounter < 2)
    ) {
      // Don't append existing spaces.
      newText += char;
    }
  }
  // Remove trailing withspaces.
  newText = newText.replace(/\s*$/, "").replace(/\s*}$/, "\n}");
  return newText;
}
