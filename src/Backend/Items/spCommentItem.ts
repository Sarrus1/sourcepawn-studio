import { CompletionItemKind, Range } from "vscode";

import { SPItem } from "./spItems";

export class CommentItem implements SPItem {
  name: string;
  filePath: string;
  kind = CompletionItemKind.User;
  range: Range;

  constructor(file: string, range: Range) {
    this.filePath = file;
    this.range = range;
  }

  toCompletionItem() {
    return undefined;
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
