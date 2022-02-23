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

import { descriptionToMD } from "../../spUtils";
import { SPItem } from "./spItems";

export class PropertyItem implements SPItem {
  parent: string;
  name: string;
  filePath: string;
  description: string;
  type: string;
  detail: string;
  kind = CompletionItemKind.Property;
  range: Range;
  references: Location[];
  fullRange: Range;

  constructor(
    parent: string,
    name: string,
    file: string,
    detail: string,
    description: string,
    range: Range,
    type: string
  ) {
    this.parent = parent;
    this.name = name;
    this.filePath = file;
    this.description = description;
    this.range = range;
    this.type = type;
    this.detail = detail;
    this.references = [];
  }

  toCompletionItem(): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: this.parent,
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
    if (!this.description) {
      return undefined;
    }
    return new Hover([
      { language: "sourcepawn", value: this.name },
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
      SymbolKind.Property,
      this.fullRange,
      this.range
    );
  }
}
