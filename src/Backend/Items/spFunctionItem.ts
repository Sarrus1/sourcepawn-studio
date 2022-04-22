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
import { SPItem } from "./spItems";
import { FunctionParam } from "../../Parser/interfaces";

export class FunctionItem implements SPItem {
  name: string;
  description: string;
  detail: string;
  params: FunctionParam[];
  filePath: string;
  range: Range;
  fullRange: Range;
  IsBuiltIn: boolean;
  references: Location[];
  kind = CompletionItemKind.Function;
  type: string;
  deprecated: string | undefined;
  accessModifiers: string[] | undefined;

  constructor(
    name: string,
    detail: string,
    description: string,
    params: FunctionParam[],
    file: string,
    IsBuiltIn: boolean,
    range: Range,
    type: string,
    fullRange: Range,
    deprecated: string | undefined,
    accessModifiers: string[] | undefined
  ) {
    this.description = description;
    this.name = name;
    this.params = params;
    this.detail = detail;
    this.filePath = file;
    this.IsBuiltIn = IsBuiltIn;
    this.range = range;
    this.type = type;
    this.fullRange = fullRange;
    this.deprecated = deprecated;
    this.references = [];
    this.accessModifiers = accessModifiers;
  }

  toCompletionItem(): CompletionItem {
    if (/\boperator\b/.test(this.name)) {
      return undefined;
    }
    return {
      label: this.name,
      kind: this.kind,
      detail: basename(this.filePath),
      tags: this.deprecated ? [CompletionItemTag.Deprecated] : [],
    };
  }

  toSignature(): SignatureInformation {
    if (/\boperator\b/.test(this.name)) {
      return undefined;
    }
    return {
      label: this.detail,
      documentation: descriptionToMD(this.description),
      parameters: this.params,
    };
  }

  toHover(): Hover {
    let filename = basename(this.filePath, ".inc");

    if (!this.description) {
      return new Hover({
        language: "sourcepawn",
        value: this.detail,
      });
    }
    if (this.IsBuiltIn) {
      return new Hover([
        {
          language: "sourcepawn",
          value: this.detail,
        },
        `[Online Documentation](https://sourcemod.dev/#/${filename}/function.${this.name})`,
        descriptionToMD(
          `${this.description}${
            this.deprecated ? `\nDEPRECATED ${this.deprecated}` : ""
          }`
        ),
      ]);
    }
    return new Hover([
      {
        language: "sourcepawn",
        value: this.detail,
      },
      descriptionToMD(
        `${this.description}${
          this.deprecated ? `\nDEPRECATED ${this.deprecated}` : ""
        }`
      ),
    ]);
  }

  toDefinitionItem(): LocationLink {
    return {
      targetRange: this.range,
      targetUri: URI.file(this.filePath),
    };
  }

  toReferenceItem(): Location[] {
    return this.references;
  }

  toDocumentSymbol(): DocumentSymbol | undefined {
    if (this.fullRange === undefined) {
      return undefined;
    }
    let name = this.name;
    const match = name.match(/\boperator\b[^\w]{1,2}/);
    if (match) {
      name = match[0];
    } else if (name === "float") {
      return undefined;
    }
    return new DocumentSymbol(
      name,
      this.description,
      SymbolKind.Function,
      this.fullRange,
      this.range
    );
  }
}
