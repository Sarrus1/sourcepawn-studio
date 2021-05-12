import * as vscode from "vscode";
import {SP_LEGENDS} from "../spLegends";

export type HighlightTokens = Map<string, vscode.Range[]>;

export class HighlightingRepository
  implements vscode.DocumentSemanticTokensProvider, vscode.Disposable {
  private globalState: vscode.Memento;
	public highlightTokens:HighlightTokens;

  constructor(globalState?: vscode.Memento) {
    this.globalState = globalState;
		this.highlightTokens = new(Map);
  }

  public dispose() {}

  public provideDocumentSemanticTokens(
    document: vscode.TextDocument
  ): vscode.ProviderResult<vscode.SemanticTokens> {
    const tokensBuilder = new vscode.SemanticTokensBuilder(SP_LEGENDS);
		for(let range of this.highlightTokens.get(document.uri.toString())){
			tokensBuilder.push(
				range,
				"class",
				["declaration"]
			);
		}
    return tokensBuilder.build();
  }
}
