import * as vscode from "vscode";

export class DocumentFormattingEditProvider
  implements vscode.DocumentFormattingEditProvider {
  public provideDocumentFormattingEdits(
    document: vscode.TextDocument,
    options: vscode.FormattingOptions,
    token: vscode.CancellationToken
  ): vscode.ProviderResult<vscode.TextEdit[]> {
    const result = [];

    const start = new vscode.Position(0, 0);
    const end = new vscode.Position(
      document.lineCount - 1,
      document.lineAt(document.lineCount - 1).text.length
    );
    const range = new vscode.Range(start, end);

    let text = this.formatText(document.getText(range));

    result.push(new vscode.TextEdit(range, text));
    return result;
  }

  private formatText(text) {
    text = text.replace(/for\s*\(/g, "for(");
		text = text.replace(/if\s*\(/g, "if(");
		text = text.replace(/else\s*\(/g, "else(");
		text = text.replace(/while\s*\(/g, "while(");
    text = text.replace(/else\s+if\s*\(/g, "else if(");
		text = text.replace(/\s*==\s*/g, " == ");
    // text = text.replace(/\s*>=\s*/g, " >= ");
    // text = text.replace(/\s*<=\s*/g, " <= ");
    // text = text.replace(/\s*\+=\s*/g, " += ");
    // text = text.replace(/\s*\-=\s*/g, " -= ");
    // text = text.replace(/\s*\*\s*/g, " * ");
    // text = text.replace(/\s*\|\|\s*/g, " || ");
    // text = text.replace(/\s*&&\s*/g, " && ");
    return text;
  }
}
