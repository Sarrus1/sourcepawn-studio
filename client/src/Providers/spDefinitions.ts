import * as vscode from "vscode";
import { URI } from "vscode-uri";

export enum DefinitionKind {
  Variable = 0,
  Function,
  Define,
  Enum,
  EnumMember,
	EnumStruct,
	EnumStructMember
}

export class DefLocation extends vscode.Location {
  type: DefinitionKind;
	scope: string;

  constructor(uri: URI, range: vscode.Range, type: DefinitionKind, scope:string = "__global") {
    super(uri, range);
    this.type = type;
		this.scope = scope;
  }
}

export type Definitions = Map<string, DefLocation>;

export class DefinitionRepository
  implements vscode.DefinitionProvider, vscode.Disposable {
  public definitions: Definitions;
  private globalState: vscode.Memento;

  constructor(globalState?: vscode.Memento) {
    this.definitions = new Map();
    this.globalState = globalState;
  }

  public provideDefinition(
    document: vscode.TextDocument,
    position: vscode.Position,
    token: vscode.CancellationToken
  ): vscode.Location | vscode.DefinitionLink[] {
    let word: string = document.getText(
      document.getWordRangeAtPosition(position)
    );
    let definition: DefLocation = this.definitions.get(word+"___gLobaL");
		if(typeof definition == "undefined"){
			let lastFuncName:string = GetLastFuncName(position.line, document);
			definition = this.definitions.get(word+"___"+lastFuncName);
		}
    if (
      typeof definition != "undefined" &&
      this.isLocalFileVariable(document, definition)
    ) {
      return new vscode.Location(definition.uri, definition.range);
    }
  }

  public dispose() {}

  public isLocalFileVariable(
    document: vscode.TextDocument,
    definition: DefLocation
  ) {
    if (definition.type === DefinitionKind.Variable) {
      return document.uri.fsPath == definition.uri.fsPath;
    }
    return true;
  }
}

function GetLastFuncName(lineNB:number, document:vscode.TextDocument):string{
	let re = /(?:static|native|stock|public|forward)?\s*(?:[a-zA-Z\-_0-9]:)?([^\s]+)\s*([A-Za-z_]*)\s*\(([^\)]*)(?:\)?)(?:\s*)(?:\{?)(?:\s*)(?:[^\;\s]*);?\s*$/;
	let text = document.getText().split("\n");
	let found=false
	let Match
	for(lineNB; lineNB>0; lineNB--){
		Match = text[lineNB].match(re)
		if(Match){
			let match = text[lineNB].match(
				/^\s*(?:(?:stock|public)\s+)*(?:(\w*)\s+)?(\w*)\s*\(([^]*)(?:\)|,|{)\s*$/
			);
			if (!match) {
				match = text[lineNB].match(
					/^\s*(?:(?:forward|static|native)\s+)+(?:(\w*)\s+)?(\w*)\s*\(([^]*)(?:,|;)\s*$/
				);
			}
			if(match&&match[1]!="if"&&match[1]!="else"&&match[1]!="while") break;
		}
	}
	if(lineNB==0) return undefined;
	let match = text[lineNB].match(re)
	// Deal with old syntax here
	return match[2]=="" ? match[1] : match[2];
}