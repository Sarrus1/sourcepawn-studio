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

export class EnumItem implements SPItem {
  name: string;
  filePath: string;
  kind = CompletionItemKind.Enum;
  description: string;
  range: Range;
  fullRange: Range;
  references: Location[];

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
    // Remove anonymous enums
    if (this.name.includes("#")) {
      return undefined;
    }
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
      { language: "sourcepawn", value: `enum ${this.name}` },
      descriptionToMD(this.description),
    ]);
  }

  toDocumentSymbol(): DocumentSymbol | undefined {
    if (this.fullRange === undefined) {
      return undefined;
    }
    return new DocumentSymbol(
      this.name.replace(/Enum#(\d+)/, "Anonymous$1"),
      this.description,
      SymbolKind.Enum,
      this.fullRange,
      this.range
    );
  }
}
