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
import { ConstantItem } from "./spConstantItem";
import { globalIdentifier, globalItem } from "../../Misc/spConstants";
import { isBuiltIn } from "../spItemsPropertyGetters";

export class MethodMapItem implements SPItem {
  name: string;
  parent: MethodMapItem | ConstantItem;
  tmpParent: string;
  description: string;
  detail: string;
  kind = CompletionItemKind.Class;
  type: string;
  range: Range;
  filePath: string;
  fullRange: Range;
  references: Location[];

  constructor(
    name: string,
    parent: string,
    description: string,
    file: string,
    range: Range,
    fullRange: Range
  ) {
    this.name = name;
    this.tmpParent = parent;
    if (parent !== null) {
      this.tmpParent = parent;
    }
    this.parent = globalItem;
    this.description = description;
    this.filePath = file;
    this.range = range;
    this.fullRange = fullRange;
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
    this.detail = `methodmap ${this.name}${
      this.parent.name !== globalIdentifier
        ? this.name + " < " + this.parent.name
        : ""
    }`;
    if (!this.description) {
      return new Hover([{ language: "sourcepawn", value: this.detail }]);
    }
    const filename: string = basename(this.filePath, ".inc");
    if (isBuiltIn(this.filePath)) {
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
