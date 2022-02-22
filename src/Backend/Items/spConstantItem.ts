import { CompletionItemKind, CompletionItem, Location } from "vscode";

import { SPItem } from "./spItems";

export class ConstantItem implements SPItem {
  name: string;
  kind = CompletionItemKind.Constant;
  references: Location[];

  constructor(name: string) {
    this.name = name;
    this.references = [];
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
