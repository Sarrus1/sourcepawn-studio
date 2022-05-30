import {
  CompletionItemKind,
  Range,
  Location,
  CompletionItem,
  Hover,
  DocumentSymbol,
  SymbolKind,
  LocationLink,
} from "vscode";
import { URI } from "vscode-uri";

import { SPItem } from "./spItems";
import { descriptionToMD } from "../../spUtils";

export class DefineItem implements SPItem {
  name: string;
  value: string;
  description?: string;
  filePath: string;
  kind = CompletionItemKind.Constant;
  IsBuiltIn: boolean;
  range: Range;
  references: Location[];
  deprecated: string | undefined;
  fullRange: Range;

  constructor(
    name: string,
    value: string,
    description: string,
    file: string,
    range: Range,
    IsBuiltIn: boolean,
    fullRange: Range,
    deprecated: string | undefined
  ) {
    this.name = name;
    this.value = value;
    this.description = description;
    this.filePath = file;
    this.range = range;
    this.references = [];
    this.IsBuiltIn = IsBuiltIn;
    this.fullRange = fullRange;
    this.deprecated = deprecated;
  }

  toCompletionItem(): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: this.filePath,
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

  toHover(): Hover {
    return new Hover([
      { language: "sourcepawn", value: `#define ${this.name} ${this.value}` },
      descriptionToMD(this.description),
    ]);
  }

  toDocumentSymbol(): DocumentSymbol | undefined {
    if (this.fullRange === undefined) {
      return undefined;
    }
    return new DocumentSymbol(
      this.name,
      this.description ? this.description.replace(/^\*\</, "") : "",
      SymbolKind.Constant,
      this.fullRange,
      this.range
    );
  }
}
