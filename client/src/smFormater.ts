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
		
		let style = "{AlignOperands: 'true',";
		style += "AlignTrailingComments: 'true',";
		style += "AllowAllArgumentsOnNextLine: 'false',";
		style += "AllowAllConstructorInitializersOnNextLine: 'false',";
		style += "AllowAllParametersOfDeclarationOnNextLine: 'false',";
		style += "AllowShortBlocksOnASingleLine: 'true',";
		style += "AllowShortCaseLabelsOnASingleLine: 'true',";
		style += "AlwaysBreakAfterDefinitionReturnType: None,";
		style += "AlwaysBreakAfterReturnType: None,";
		style += "AlwaysBreakBeforeMultilineStrings: 'false',";
		style += "BinPackArguments: 'false',";
		style += "BinPackParameters: 'false',";
		style += "BreakBeforeBraces: Allman,";
		style += "BreakBeforeTernaryOperators: 'false',";
		style += "BreakStringLiterals: 'false',";
		style += "ColumnLimit: '0',";
		style += "ContinuationIndentWidth: '2',";
		style += "IndentWidth: '2',";
		style += "MaxEmptyLinesToKeep: '2',";
		style += "SpaceAfterLogicalNot: 'false',";
		style += "SpaceBeforeParens: Never,";
		style += "SpaceBeforeRangeBasedForLoopColon: 'false',";
		style += "SpaceInEmptyParentheses: 'false',";
		style += "TabWidth: '2',";
		style += "UseTab: Always}";


    const start = new vscode.Position(0, 0);
    const end = new vscode.Position(
      document.lineCount - 1,
      document.lineAt(document.lineCount - 1).text.length
    );
    const range = new vscode.Range(start, end);
		let text:string = clangFormat(document, 'utf-8', style, this.Callback);
		
		if(text === "") return;
		text = text.replace(/^ *public\s*\n/gm, "public ")
    result.push(new vscode.TextEdit(range, text));
    return result;
  }
}
