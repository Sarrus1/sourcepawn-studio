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

  toCompletionItem(): CompletionItem {
    return;
  }

  toDefinitionItem(): LocationLink {
    return;
  }

  toSignature(): SignatureInformation {
    return;
  }

  toHover(): Hover {
    return;
  }

  toDocumentSymbol(): DocumentSymbol {
    return;
  }
}
