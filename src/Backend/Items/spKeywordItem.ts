import {
  CompletionItemKind,
  CompletionItem,
  SignatureInformation,
  Hover,
  DocumentSymbol,
  LocationLink,
} from "vscode";

import { SPItem } from "./spItems";

export class KeywordItem implements SPItem {
  name: string;
  kind = CompletionItemKind.Keyword;

  constructor(name: string) {
    this.name = name;
  }

  toCompletionItem(): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: "",
    };
  }

  toDefinitionItem() {
    return undefined;
  }

  toSignature() {
    return undefined;
  }

  toHover() {
    return undefined;
  }

  toDocumentSymbol() {
    return undefined;
  }
}
