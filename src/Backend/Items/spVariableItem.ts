import {
  CompletionItemKind,
  Range,
  CompletionItem,
  Hover,
  DocumentSymbol,
  SymbolKind,
  LocationLink,
  Location,
  Position,
} from "vscode";
import { URI } from "vscode-uri";

import { SPItem } from "./spItems";
import { globalIdentifier } from "../../Misc/spConstants";
import { ConstantItem } from "./spConstantItem";
import { MethodItem } from "./spMethodItem";
import { FunctionItem } from "./spFunctionItem";
import { descriptionToMD } from "../../spUtils";
import { EnumStructItem } from "./spEnumStructItem";
import { MethodMapItem } from "./spMethodmapItem";

export class VariableItem implements SPItem {
  name: string;
  filePath: string;
  detail: string;
  kind = CompletionItemKind.Variable;
  description?: string;
  parent: SPItem | ConstantItem;
  range: Range;
  type: string;
  references: Location[];

  constructor(
    name: string,
    file: string,
    parent: SPItem | ConstantItem,
    range: Range,
    type: string,
    detail: string,
    description = ""
  ) {
    this.name = name;
    this.filePath = file;
    this.parent = parent;
    this.range = range;
    this.type = type;
    this.references = [];
    this.detail = detail;
    this.description = description;
  }

  toCompletionItem(
    lastFunc: MethodItem | FunctionItem | undefined,
    lastMMorES: MethodMapItem | EnumStructItem | undefined,
    position?: Position,
    override?: boolean
  ): CompletionItem | undefined {
    if (override || this.parent.name === globalIdentifier) {
      return {
        label: this.name,
        kind: this.kind,
      };
    }

    if (lastFunc === undefined) {
      return undefined;
    }

    if (lastMMorES === undefined) {
      if (
        lastFunc.name === this.parent.name &&
        (this.range.end.line < position.line ||
          this.range.end.character < position.character)
      ) {
        return {
          label: this.name,
          kind: this.kind,
        };
      }
      return undefined;
    }

    lastFunc = lastFunc as MethodItem;
    if (
      this.parent.name === lastFunc.name &&
      lastFunc.parent.name === lastMMorES.name
    ) {
      return {
        label: this.name,
        kind: this.kind,
      };
    }

    return undefined;
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
    if (this.detail === "") {
      return undefined;
    }
    return new Hover([
      { language: "sourcepawn", value: this.detail.trim() },
      descriptionToMD(this.description),
    ]);
  }

  toDocumentSymbol(): DocumentSymbol {
    return new DocumentSymbol(
      this.name,
      this.type,
      SymbolKind.Variable,
      this.range,
      this.range
    );
  }
}
