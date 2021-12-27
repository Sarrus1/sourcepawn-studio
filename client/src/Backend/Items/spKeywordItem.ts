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
  commitCharacters = [";", ":", "<", "("];

  constructor(name: string) {
    this.name = name;
  }

  toCompletionItem(file: string, lastFuncName?: string): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: "",
      commitCharacters: this.commitCharacters,
    };
  }

  toDefinitionItem(): LocationLink {
    return undefined;
  }

  toSignature(): SignatureInformation {
    return undefined;
  }

  toHover(): Hover {
    return undefined;
  }

  toDocumentSymbol(): DocumentSymbol {
    return undefined;
  }
}
