import {
  CompletionItemKind,
  Range,
  Location,
  CompletionItem,
  SignatureInformation,
  Hover,
  DocumentSymbol,
  SymbolKind,
  LocationLink,
} from "vscode";
import { URI } from "vscode-uri";

import { SPItem } from "./spItems";

export class DefineItem implements SPItem {
  name: string;
  value: string;
  file: string;
  kind = CompletionItemKind.Constant;
  IsBuiltIn: boolean;
  range: Range;
  calls: Location[];
  fullRange: Range;
  commitCharacters = [";", ","];

  constructor(
    name: string,
    value: string,
    file: string,
    range: Range,
    IsBuiltIn: boolean,
    fullRange: Range
  ) {
    this.name = name;
    this.value = value;
    this.file = file;
    this.range = range;
    this.calls = [];
    this.IsBuiltIn = IsBuiltIn;
    this.fullRange = fullRange;
  }

  toCompletionItem(): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: this.file,
      commitCharacters: this.commitCharacters,
    };
  }

  toDefinitionItem(): LocationLink {
    return {
      targetRange: this.range,
      targetUri: URI.file(this.file),
    };
  }

  toSignature(): SignatureInformation {
    return;
  }

  toHover(): Hover {
    return new Hover({
      language: "sourcepawn",
      value: `#define ${this.name} ${this.value}`,
    });
  }

  toDocumentSymbol(): DocumentSymbol {
    if (this.fullRange === undefined) {
      return;
    }
    return new DocumentSymbol(
      this.name,
      `#define ${this.name} ${this.value}`,
      SymbolKind.Constant,
      this.fullRange,
      this.range
    );
  }
}
