import {
  CompletionItemKind,
  Range,
  CompletionItem,
  Hover,
  DocumentSymbol,
  SymbolKind,
  LocationLink,
  Location,
  SnippetString,
} from "vscode";
import { URI } from "vscode-uri";
import { basename } from "path";

import { descriptionToMD } from "../../spUtils";
import { SPItem } from "./spItems";
import { FormalParameter } from "../../Parser/interfaces";

export class TypedefItem implements SPItem {
  name: string;
  details: string;
  type: string;
  filePath: string;
  description: string;
  kind = CompletionItemKind.TypeParameter;
  range: Range;
  fullRange: Range;
  references: Location[];
  params_signature: FormalParameter[];

  constructor(
    name: string,
    details: string,
    file: string,
    description: string,
    type: string,
    range: Range,
    fullRange: Range,
    params_signature: FormalParameter[]
  ) {
    this.name = name;
    this.details = details;
    this.filePath = file;
    this.type = type;
    this.description = description;
    this.range = range;
    this.fullRange = fullRange;
    this.references = [];
    this.params_signature = params_signature;
  }

  toCompletionItem(): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: basename(this.filePath),
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

  toHover(): Hover | undefined {
    return new Hover([
      {
        language: "sourcepawn",
        value: this.details,
      },
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
      SymbolKind.TypeParameter,
      this.fullRange,
      this.range
    );
  }

  toSnippet(range: Range): CompletionItem {
    const snippet = new SnippetString();
    snippet.appendText(`public ${this.type} `);
    snippet.appendPlaceholder("name");
    snippet.appendText("(");
    if (this.params_signature) {
      this.params_signature.forEach((param, i) => {
        let declarationType = Array.isArray(param.declarationType)
          ? param.declarationType.join(" ")
          : param.declarationType;
        if (declarationType) {
          snippet.appendText(declarationType);
          snippet.appendText(" ");
        }
        let type = param.parameterType;
        if (type) {
          snippet.appendText(type.name.id);
          snippet.appendText(type.modifier);
        }
        snippet.appendPlaceholder(param.id.id);
        if (i !== this.params_signature.length - 1) {
          snippet.appendText(", ");
        }
      });
    }

    snippet.appendText(")\n{\n\t");
    snippet.appendTabstop();
    snippet.appendText("\n}");
    return {
      label: this.name,
      filterText: "$" + this.name,
      range,
      kind: CompletionItemKind.Function,
      insertText: snippet,
      detail: this.details,
    };
  }
}
