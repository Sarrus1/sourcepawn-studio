import {
  CompletionItemKind,
  Range,
  CompletionItem,
  Hover,
  DocumentSymbol,
  SymbolKind,
  LocationLink,
  Location,
} from "vscode";
import { URI } from "vscode-uri";
import { basename } from "path";

import { descriptionToMD } from "../../spUtils";
import { SPItem } from "./spItems";

export class EnumStructItem implements SPItem {
  name: string;
  filePath: string;
  description: string;
  kind = CompletionItemKind.Struct;
  references: Location[];
  range: Range;
  fullRange: Range;

  constructor(
    name: string,
    file: string,
    description: string,
    range: Range,
    fullRange: Range
  ) {
    this.name = name;
    this.filePath = file;
    this.description = description;
    this.range = range;
    this.fullRange = fullRange;
    this.references = [];
  }

  toCompletionItem(): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: basename(this.filePath),
    };
  }

  toDefinitionItem(): LocationLink {
    return {
      targetRange: this.range,
      targetUri: URI.file(this.filePath),
    };
  }

  toSignature() {
    return undefined;
  }

  toHover(): Hover | undefined {
    return new Hover([
      { language: "sourcepawn", value: `enum struct ${this.name}` },
      descriptionToMD(this.description),
    ]);
  }

  toDocumentSymbol(): DocumentSymbol | undefined {
    if (this.fullRange === undefined) {
      return undefined;
    }
    return new DocumentSymbol(
      this.name,
      this.description,
      SymbolKind.Struct,
      this.fullRange,
      this.range
    );
  }
}
