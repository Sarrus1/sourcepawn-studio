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

export class MethodMapItem implements SPItem {
  name: string;
  parent: string;
  description: string;
  detail: string;
  kind = CompletionItemKind.Class;
  type: string;
  range: Range;
  IsBuiltIn: boolean;
  filePath: string;
  fullRange: Range;
  references: Location[];

  constructor(
    name: string,
    parent: string,
    detail: string,
    description: string,
    file: string,
    range: Range,
    IsBuiltIn: boolean = false
  ) {
    this.name = name;
    this.parent = parent;
    this.detail = detail;
    this.description = description;
    this.IsBuiltIn = IsBuiltIn;
    this.filePath = file;
    this.range = range;
    this.type = name;
    this.references = [];
  }

  toCompletionItem(): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: basename(this.filePath, ".inc"),
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
    if (!this.description) {
      return new Hover([{ language: "sourcepawn", value: this.detail }]);
    }
    let filename: string = basename(this.filePath, ".inc");
    if (this.IsBuiltIn) {
      return new Hover([
        { language: "sourcepawn", value: this.detail },
        `[Online Documentation](https://sourcemod.dev/#/${filename}/methodmap.${this.name})`,
        descriptionToMD(this.description),
      ]);
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
      SymbolKind.Class,
      this.fullRange,
      this.range
    );
  }
}