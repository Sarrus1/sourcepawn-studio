import {
  CompletionItemKind,
  Range,
  CompletionItem,
  SignatureInformation,
  Hover,
  DocumentSymbol,
  SymbolKind,
  LocationLink,
  CompletionItemTag,
  Location,
} from "vscode";
import { URI } from "vscode-uri";
import { basename } from "path";

import { descriptionToMD } from "../../spUtils";
import { SPItem, FunctionParam } from "./spItems";

export class MethodItem implements SPItem {
  name: string;
  parent: string;
  description: string;
  detail: string;
  params: FunctionParam[];
  kind: CompletionItemKind;
  fullRange: Range;
  type: string;
  range: Range;
  IsBuiltIn: boolean;
  filePath: string;
  references: Location[];
  deprecated: string | undefined;

  constructor(
    parent: string,
    name: string,
    detail: string,
    description: string,
    params: FunctionParam[],
    type: string,
    file: string,
    range: Range,
    IsBuiltIn: boolean = false,
    fullRange: Range,
    deprecated: string | undefined
  ) {
    this.parent = parent;
    this.name = name;
    this.kind =
      this.name == this.parent
        ? CompletionItemKind.Constructor
        : CompletionItemKind.Method;
    this.detail = detail;
    this.description = description;
    this.params = params;
    this.type = type;
    this.IsBuiltIn = IsBuiltIn;
    this.filePath = file;
    this.range = range;
    this.fullRange = fullRange;
    this.deprecated = deprecated;
    this.references = [];
  }

  toCompletionItem(): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: this.parent,
      tags: this.deprecated ? [CompletionItemTag.Deprecated] : [],
    };
  }

  toDefinitionItem(): LocationLink {
    return {
      targetRange: this.range,
      targetUri: URI.file(this.filePath),
    };
  }

  toSignature(): SignatureInformation {
    return {
      label: this.detail,
      documentation: descriptionToMD(this.description),
      parameters: this.params,
    };
  }

  toHover(): Hover {
    if (!this.description) {
      return new Hover([{ language: "sourcepawn", value: this.detail }]);
    }
    let filename: string = basename(this.filePath, ".inc");
    if (this.IsBuiltIn) {
      return new Hover([
        { language: "sourcepawn", value: this.detail },
        `[Online Documentation](https://sourcemod.dev/#/${filename}/methodmap.${this.parent}/function.${this.name})`,
        descriptionToMD(
          `${this.description}${
            this.deprecated ? `\nDEPRECATED ${this.deprecated}` : ""
          }`
        ),
      ]);
    }
    return new Hover([
      { language: "sourcepawn", value: this.detail },
      descriptionToMD(
        `${this.description}${
          this.deprecated ? `\nDEPRECATED ${this.deprecated}` : ""
        }`
      ),
    ]);
  }

  toDocumentSymbol(): DocumentSymbol | undefined {
    if (this.fullRange === undefined) {
      return undefined;
    }
    return new DocumentSymbol(
      this.name,
      this.description,
      SymbolKind.Method,
      this.fullRange,
      this.range
    );
  }
}
