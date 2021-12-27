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
import { basename } from "path";

import { descriptionToMD } from "../../spUtils";
import { SPItem } from "./spItems";

export class EnumStructItem implements SPItem {
  name: string;
  filePath: string;
  description: string;
  kind = CompletionItemKind.Struct;
  range: Range;
  fullRange: Range;
  commitCharacters = [";"];

  constructor(name: string, file: string, description: string, range: Range) {
    this.name = name;
    this.filePath = file;
    this.description = description;
    this.range = range;
  }

  toCompletionItem(): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: basename(this.filePath),
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
      SymbolKind.Struct,
      this.fullRange,
      this.range
    );
  }
}
