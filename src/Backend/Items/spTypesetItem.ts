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
import { TypedefItem } from "./spTypedefItem";

export class TypesetItem implements SPItem {
  name: string;
  details: string;
  filePath: string;
  description: string;
  kind = CompletionItemKind.TypeParameter;
  range: Range;
  fullRange: Range;
  references: Location[];
  childs: TypedefItem[];

  constructor(
    name: string,
    details: string,
    file: string,
    description: string,
    range: Range,
    fullRange: Range,
    childs: TypedefItem[]
  ) {
    this.name = name;
    this.details = details;
    this.filePath = file;
    this.description = description;
    this.range = range;
    this.fullRange = fullRange;
    this.references = [];
    this.childs = childs;
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
      { language: "sourcepawn", value: this.details },
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
      SymbolKind.TypeParameter,
      this.fullRange,
      this.range
    );
  }

  toSnippet(range: Range): CompletionItem[] {
    return this.childs.map((e) => e.toSnippet(range));
  }
}
