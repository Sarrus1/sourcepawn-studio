import {
  CompletionItemKind,
  Range,
  CompletionItem,
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
import { ConstantItem } from "./spConstantItem";
import { globalItem } from "../../Misc/spConstants";

export class EnumMemberItem implements SPItem {
  name: string;
  parent: EnumItem | ConstantItem;
  filePath: string;
  description: string;
  kind = CompletionItemKind.EnumMember;
  range: Range;
  references: Location[];

  constructor(
    name: string,
    file: string,
    range: Range,
    enumItem: EnumItem | ConstantItem = globalItem
  ) {
    this.name = name;
    this.filePath = file;
    this.description = "";
    this.range = range;
    this.references = [];
    this.parent = enumItem;
  }

  toCompletionItem(): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail:
        this.parent === globalItem ? basename(this.filePath) : this.parent.name,
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
    const enumName = this.parent !== globalItem ? this.parent.name : "";
    return new Hover([
      { language: "sourcepawn", value: `${enumName} ${this.name};` },
      descriptionToMD(this.description),
    ]);
  }

  toDocumentSymbol(): DocumentSymbol | undefined {
    if (this.name === "") {
      return undefined;
    }
    return new DocumentSymbol(
      this.name,
      this.description.replace(/^\*\</, ""),
      SymbolKind.Enum,
      this.range,
      this.range
    );
  }
}
