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

import { SPItem } from "./spItems";
import { globalIdentifier } from "../../Misc/spConstants";
import { ConstantItem } from "./spConstantItem";
import { MethodItem } from "./spMethodItem";
import { FunctionItem } from "./spFunctionItem";
import { descriptionToMD } from "../../spUtils";
import { EnumStructItem } from "./spEnumStructItem";
import { MethodMapItem } from "./spMethodmapItem";
import { PropertyItem } from "./spPropertyItem";

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
  accessModifiers: string[] | undefined;

  constructor(
    name: string,
    file: string,
    parent: SPItem | ConstantItem,
    range: Range,
    type: string,
    detail: string,
    description: string,
    accessModifiers: string[] | undefined
  ) {
    this.name = name;
    this.filePath = file;
    this.parent = parent;
    this.range = range;
    this.type = type;
    this.references = [];
    this.detail = detail;
    this.description = description;
    this.accessModifiers = accessModifiers ? accessModifiers : undefined;
  }

  toCompletionItem(
    lastFunc: MethodItem | FunctionItem | undefined,
    lastMMorES: MethodMapItem | EnumStructItem | undefined,
    location?: Location,
    override?: boolean
  ): CompletionItem | undefined {
    if (
      this.accessModifiers !== undefined &&
      location &&
      this.accessModifiers.includes("static") &&
      location.uri.fsPath !== this.filePath
    ) {
      // static global variables should not appear outside of their file.
      return undefined;
    }

    if (override || this.parent.name === globalIdentifier) {
      return {
        label: this.name,
        kind: this.kind,
      };
    }

    if (this.filePath !== location.uri.fsPath || lastFunc === undefined) {
      return undefined;
    }

    if (lastMMorES === undefined) {
      if (
        lastFunc.name === this.parent.name &&
        this.range.end.isBeforeOrEqual(location.range.start)
      ) {
        return {
          label: this.name,
          kind: this.kind,
        };
      }
      return undefined;
    }

    lastFunc = lastFunc as MethodItem;
    const parent = this.parent as MethodItem;
    if (
      lastFunc.fullRange.contains(this.range) &&
      lastFunc.parent.fullRange.contains(this.range)
    ) {
      if (
        parent.parent.kind === CompletionItemKind.Property &&
        (parent.parent as PropertyItem).parent.name === lastMMorES.name
      ) {
        return {
          label: this.name,
          kind: this.kind,
        };
      } else if (parent.kind === CompletionItemKind.Method) {
        return {
          label: this.name,
          kind: this.kind,
        };
      }
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
