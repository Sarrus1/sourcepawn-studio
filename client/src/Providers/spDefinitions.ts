import * as vscode from "vscode";
import { URI } from "vscode-uri";

export enum DefinitionKind {
  Variable = 0,
  Function,
  Define,
  Enum,
  EnumMember,
  EnumStruct,
  EnumStructMember,
}

export class DefLocation extends vscode.Location {
  type: DefinitionKind;
  scope: string;

  constructor(
    uri: URI,
    range: vscode.Range,
    type: DefinitionKind,
    scope: string = "___GLOBALLL"
  ) {
    super(uri, range);
    this.type = type;
    this.scope = scope;
  }
}

export type Definitions = Map<string, DefLocation>;

export class DefinitionRepository
  implements vscode.DefinitionProvider, vscode.Disposable {
  public otherDefinitions: Definitions;
  public functionDefinitions: Definitions;
  private globalState: vscode.Memento;

  constructor(globalState?: vscode.Memento) {
    this.otherDefinitions = new Map();
    this.functionDefinitions = new Map();
    this.globalState = globalState;
  }

  public provideDefinition(
    document: vscode.TextDocument,
    position: vscode.Position,
    token: vscode.CancellationToken
  ): vscode.Location | vscode.DefinitionLink[] {
    let range = document.getWordRangeAtPosition(position);
    let word: string = document.getText(range);
    let definition: DefLocation;
    let isFunction = this.isFunction(
      range,
      document,
      document.getText().split("\n")[position.line].length
    );
    if (isFunction) {
      definition = this.functionDefinitions.get(word + "___GLOBALLL");
      if (
        typeof definition != "undefined" &&
        this.isLocalFileVariable(document, definition)
      ) {
        return new vscode.Location(definition.uri, definition.range);
      }
    }
    definition = this.otherDefinitions.get(word + "___GLOBALLL");
    if (typeof definition == "undefined") {
      let lastFuncName: string = GetLastFuncName(position.line, document);
      definition = this.otherDefinitions.get(word + "___" + lastFuncName);
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

  public isFunction(
    range: vscode.Range,
    document: vscode.TextDocument,
    lineLength: number
  ): boolean {
    let start = new vscode.Position(range.start.line, range.end.character);
    let end = new vscode.Position(range.end.line, lineLength + 1);
    let rangeAfter = new vscode.Range(start, end);
    let wordsAfter: string = document.getText(rangeAfter);
    return /^\s*\(/.test(wordsAfter);
  }
}

function GetLastFuncName(
  lineNB: number,
  document: vscode.TextDocument
): string {
  let re = /(?:static|native|stock|public|forward)?\s*(?:[a-zA-Z\-_0-9]:)?([^\s]+)\s*([A-Za-z_]*)\s*\(([^\)]*)(?:\)?)(?:\s*)(?:\{?)(?:\s*)(?:[^\;\s]*);?\s*$/;
  let text = document.getText().split("\n");
  let found = false;
  let Match;
  let line: string;
  for (lineNB; lineNB > 0; lineNB--) {
    line = text[lineNB];
    Match = line.match(re);
    if (Match) {
      let match = line.match(
        /^\s*(?:(?:stock|public)\s+)*(?:(\w*)\s+)?(\w*)\s*\(([^]*)(?:\)|,|{)\s*$/
      );
      if (!match) {
        match = line.match(
          /^\s*(?:(?:forward|static|native)\s+)+(?:(\w*)\s+)?(\w*)\s*\(([^]*)(?:,|;)\s*$/
        );
      }
      if (match && CheckIfControlStatement(line)) break;
    }
  }
  if (lineNB == 0) return undefined;
  let match = text[lineNB].match(re);
  // Deal with old syntax here
  return match[2] == "" ? match[1] : match[2];
}

function CheckIfControlStatement(line: string): boolean {
  let toCheck: RegExp[] = [
    /\s*\bif\b/,
		/\s*\bfor\b/,
    /\s*\bwhile\b/,
    /\s*\bcase\b/,
    /\s*\bswitch\b/,
  ];
  for (let re of toCheck) {
    if (re.test(line)) {
      return false;
    }
  }
  return true;
}
