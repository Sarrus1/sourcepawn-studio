import * as vscode from "vscode";
import * as glob from "glob";
import * as path from "path";
import { URI } from "vscode-uri";
import * as fs from "fs";

export type Definitions = Map<string, vscode.Location>;

export class DefinitionRepository implements vscode.DefinitionProvider, vscode.Disposable {
  public definitions: Definitions;
  documents: Set<vscode.Uri>;
  private globalState: vscode.Memento;

  constructor(globalState?: vscode.Memento) {
    this.definitions = new Map();
    this.documents = new Set();
    this.globalState = globalState;
  }

  public provideDefinition(document: vscode.TextDocument, position: vscode.Position, token: vscode.CancellationToken): vscode.Location | vscode.DefinitionLink[]{
		let word : string = document.getText(document.getWordRangeAtPosition(position));
		let definition : vscode.Location = this.definitions.get(word);
		if(typeof definition != "undefined")
		{
			return new vscode.Location(definition.uri , definition.range);
		}
	};

	public dispose() {}

}