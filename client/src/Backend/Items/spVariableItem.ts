import {
  CompletionItemKind,
  Range,
  CompletionItem,
  SignatureInformation,
  Hover,
  DocumentSymbol,
  SymbolKind,
  LocationLink,
} from "vscode";
import { URI } from "vscode-uri";

import { SPItem } from "./spItems";
import { globalIdentifier } from "../../Misc/spConstants";

export class VariableItem implements SPItem {
  name: string;
  file: string;
  kind = CompletionItemKind.Variable;
  parent: string;
  range: Range;
  type: string;
  enumStructName: string;
  commitCharacters = [";", "."];

  constructor(
    name: string,
    file: string,
    parent: string,
    range: Range,
    type: string,
    enumStruct: string
  ) {
    this.name = name;
    this.file = file;
    this.parent = parent;
    this.range = range;
    this.type = type;
    this.enumStructName = enumStruct;
  }

  toCompletionItem(file: string, lastFuncName?: string): CompletionItem {
    if (lastFuncName !== undefined) {
      if (this.parent === lastFuncName) {
        return {
          label: this.name,
          kind: this.kind,
          commitCharacters: this.commitCharacters,
        };
      } else if (this.parent === globalIdentifier) {
        return {
          label: this.name,
          kind: this.kind,
          commitCharacters: this.commitCharacters,
        };
      }
      return undefined;
    } else {
      return {
        label: this.name,
        kind: this.kind,
        commitCharacters: this.commitCharacters,
      };
    }
  }

  toDefinitionItem(): LocationLink {
    return {
      targetRange: this.range,
      targetUri: URI.file(this.file),
    };
  }

  toSignature(): SignatureInformation {
    return undefined;
  }

  toHover(): Hover {
    return;
  }

  toDocumentSymbol(): DocumentSymbol {
    return new DocumentSymbol(
      this.name,
      this.type,
      SymbolKind.Variable,
      this.range,
      this.range
    );
  }
}
