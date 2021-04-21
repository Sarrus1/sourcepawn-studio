import * as vscode from "vscode";
import { URI } from "vscode-uri";

export enum DefinitionKind {
  Variable = 0,
  Function = 1,
  Define = 2,
  Enum = 3,
  EnumMember = 4,
}

export class DefLocation extends vscode.Location {
  type: DefinitionKind;

  constructor(uri: URI, range: vscode.Range, type: DefinitionKind) {
    super(uri, range);
    this.type = type;
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
    let definition: DefLocation = this.definitions.get(word);
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
