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
  file: string;
  description: string;
  kind = CompletionItemKind.Struct;
  range: Range;
  fullRange: Range;
  commitCharacters = [";"];

  constructor(name: string, file: string, description: string, range: Range) {
    this.name = name;
    this.file = file;
    this.description = description;
    this.range = range;
  }

  toCompletionItem(file: string, lastFuncName?: string): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: basename(this.file),
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
    return undefined;
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
