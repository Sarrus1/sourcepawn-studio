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
import { SPItem, FunctionParam } from "./spItems";

export class FunctionItem implements SPItem {
  name: string;
  description: string;
  detail: string;
  params: FunctionParam[];
  filePath: string;
  range: Range;
  fullRange: Range;
  IsBuiltIn: boolean;
  kind = CompletionItemKind.Function;
  type: string;
  commitCharacters = [";", "(", ","];

  constructor(
    name: string,
    detail: string,
    description: string,
    params: FunctionParam[],
    file: string,
    IsBuiltIn: boolean,
    range: Range,
    type: string,
    fullRange: Range
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
  }

  toCompletionItem(): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: basename(this.filePath),
      commitCharacters: this.commitCharacters,
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
    let filename: string = basename(this.filePath, ".inc");
    if (this.description == "") {
      return new Hover({ language: "sourcepawn", value: this.detail });
    }
    if (this.IsBuiltIn) {
      return new Hover([
        { language: "sourcepawn", value: this.detail },
        `[Online Documentation](https://sourcemod.dev/#/${filename}/function.${this.name})`,
        descriptionToMD(this.description),
      ]);
    }
    return new Hover([
      { language: "sourcepawn", value: this.detail },
      descriptionToMD(this.description),
    ]);
  }

  toDefinitionItem(): LocationLink {
    return {
      targetRange: this.range,
      targetUri: URI.file(this.filePath),
    };
  }

  toDocumentSymbol(): DocumentSymbol {
    if (this.fullRange === undefined) {
      return;
    }
    return new DocumentSymbol(
      this.name,
      this.description,
      SymbolKind.Function,
      this.fullRange,
      this.range
    );
  }
}
