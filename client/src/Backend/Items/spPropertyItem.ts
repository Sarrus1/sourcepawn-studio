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
  fullRange: Range;
  commitCharacters = [";", "."];

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
  }

  toCompletionItem(): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: this.parent,
      commitCharacters: this.commitCharacters,
    };
  }

  toDefinitionItem(): LocationLink {
    return {
      targetRange: this.range,
      targetUri: URI.file(this.filePath),
    };
  }

  toSignature(): SignatureInformation {
    return;
  }

  toHover(): Hover {
    if (!this.description) {
      return;
    }
    return new Hover([
      { language: "sourcepawn", value: this.name },
      descriptionToMD(this.description),
    ]);
  }

  toDocumentSymbol(): DocumentSymbol {
    if (this.fullRange === undefined) {
      return;
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
