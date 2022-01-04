import { CompletionItemKind } from "vscode";

import { FunctionItem } from "./spFunctionItem";

export class MacroItem extends FunctionItem {
  kind = CompletionItemKind.Interface;
}
