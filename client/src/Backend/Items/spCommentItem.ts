import {
  CompletionItemKind,
  Range,
  CompletionItem,
  SignatureInformation,
  Hover,
  DocumentSymbol,
  LocationLink,
} from "vscode";

import { SPItem } from "./spItems";

export class CommentItem implements SPItem {
  name: string;
  file: string;
  kind = CompletionItemKind.User;
  range: Range;

  constructor(file: string, range: Range) {
    this.file = file;
    this.range = range;
  }

  toCompletionItem(file: string, lastFuncName?: string): CompletionItem {
    return undefined;
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
