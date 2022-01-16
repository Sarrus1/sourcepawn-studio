import {
  CompletionItemKind,
  Range,
  CompletionItem,
  SignatureInformation,
  Hover,
  Location,
  DocumentSymbol,
  SymbolKind,
  LocationLink,
} from "vscode";
import { URI } from "vscode-uri";
import { basename } from "path";

import { descriptionToMD } from "../../spUtils";
import { EnumItem } from "./spEnumItem";
import { SPItem } from "./spItems";

export class EnumMemberItem implements SPItem {
  name: string;
  parent: string;
  filePath: string;
  description: string;
  kind = CompletionItemKind.EnumMember;
  range: Range;
  calls: Location[];
  IsBuiltIn: boolean;

  constructor(
    name: string,
    file: string,
    description: string,
    Enum: EnumItem,
    range: Range,
    IsBuiltItn: boolean
  ) {
    this.name = name;
    this.filePath = file;
    this.description = description;
    this.range = range;
    this.calls = [];
    this.IsBuiltIn = IsBuiltItn;
    this.parent = Enum.name;
  }

  toCompletionItem(): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: this.parent === "" ? basename(this.filePath) : this.parent,
    };
  }

  toDefinitionItem(): LocationLink {
    return {
      targetRange: this.range,
      targetUri: URI.file(this.filePath),
    };
  }

  toSignature(): SignatureInformation {
    return undefined;
  }

  toHover(): Hover {
    let enumName = this.parent.replace(/Enum#(\d+)/, "Anonymous$1");
    return new Hover([
      { language: "sourcepawn", value: `${enumName} ${this.name};` },
      descriptionToMD(this.description),
    ]);
  }

  toDocumentSymbol(): DocumentSymbol {
    if (this.name === "") {
      return;
    }
    return new DocumentSymbol(
      this.name,
      this.description,
      SymbolKind.Enum,
      this.range,
      this.range
    );
  }
}
