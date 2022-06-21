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
import { MethodMapItem } from "./spMethodmapItem";

export class PropertyItem implements SPItem {
  parent: MethodMapItem;
  name: string;
  filePath: string;
  description: string;
  type: string;
  detail: string;
  kind = CompletionItemKind.Property;
  range: Range;
  references: Location[];
  fullRange: Range;
  deprecated: string;

  constructor(
    parent: MethodMapItem,
    name: string,
    file: string,
    detail: string,
    description: string,
    range: Range,
    fullRange: Range,
    type: string,
    deprecated: string | undefined
  ) {
    this.parent = parent;
    this.name = name;
    this.filePath = file;
    this.description = description;
    this.range = range;
    this.fullRange = fullRange;
    this.type = type;
    this.detail = detail;
    this.references = [];
    this.deprecated = deprecated;
  }

  toCompletionItem(): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: this.parent.name,
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
      { language: "sourcepawn", value: this.detail },
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
