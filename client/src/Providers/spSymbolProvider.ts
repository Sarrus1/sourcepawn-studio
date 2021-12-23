import {
  TextDocument,
  CancellationToken,
  DocumentSymbol,
  CompletionItemKind,
} from "vscode";
import { ItemsRepository } from "./spItemsRepository";
import { globalIdentifier } from "../Misc/spConstants";

const allowedKinds = [
  CompletionItemKind.Function,
  CompletionItemKind.Class,
  CompletionItemKind.Struct,
  CompletionItemKind.Enum,
  CompletionItemKind.Constant,
  CompletionItemKind.Variable,
  CompletionItemKind.TypeParameter,
];
const allowedParentsKinds = [
  CompletionItemKind.Class,
  CompletionItemKind.Struct,
  CompletionItemKind.Function,
  CompletionItemKind.Enum,
];
const allowedChildrendKinds = [
  CompletionItemKind.Method,
  CompletionItemKind.Property,
  CompletionItemKind.Variable,
  CompletionItemKind.EnumMember,
];

export function symbolProvider(
  itemsRepo: ItemsRepository,
  document: TextDocument,
  token: CancellationToken
): DocumentSymbol[] {
  let symbols: DocumentSymbol[] = [];
  let items = itemsRepo.getAllItems(document.uri);
  let file = document.uri.fsPath;
  for (let item of items) {
    if (allowedKinds.includes(item.kind) && item.file === file) {
      // Don't add non global variables here
      if (
        item.kind === CompletionItemKind.Variable &&
        item.parent !== globalIdentifier
      ) {
        continue;
      }
      let symbol = item.toDocumentSymbol();

      // Check if the item can have childrens
      if (allowedParentsKinds.includes(item.kind) && symbol !== undefined) {
        symbol.children = items
          .filter(
            (e) =>
              allowedChildrendKinds.includes(e.kind) &&
              e.file === file &&
              e.parent === item.name
          )
          .map((e) => e.toDocumentSymbol())
          .filter((e) => e !== undefined);
      }
      if (symbol !== undefined) {
        symbols.push(symbol);
      }
    }
  }
  return symbols;
}
