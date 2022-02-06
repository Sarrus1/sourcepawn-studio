import {
  TextDocument,
  CompletionItemKind,
  SemanticTokens,
  SemanticTokensBuilder,
} from "vscode";
import { ItemsRepository } from "../Backend/spItemsRepository";
import { SP_LEGENDS } from "../Misc/spConstants";
import { SPItem } from "../Backend/Items/spItems";

export function semanticTokenProvider(
  itemsRepo: ItemsRepository,
  document: TextDocument
): SemanticTokens {
  const tokensBuilder = new SemanticTokensBuilder(SP_LEGENDS);
  let allItems: SPItem[] = itemsRepo.getAllItems(document.uri);
  for (let item of allItems) {
    if (item.kind === CompletionItemKind.Constant) {
      for (let call of item.calls) {
        if (call.uri.fsPath === document.uri.fsPath) {
          if (item.range.contains(call.range)) {
            tokensBuilder.push(call.range, "variable", ["declaration"]);
          } else {
            tokensBuilder.push(call.range, "variable", ["readonly"]);
          }
        }
      }
    } else if (item.kind === CompletionItemKind.EnumMember) {
      for (let call of item.calls) {
        if (call.uri.fsPath === document.uri.fsPath) {
          if (item.range.contains(call.range)) {
            tokensBuilder.push(call.range, "enumMember", ["declaration"]);
          } else {
            tokensBuilder.push(call.range, "enumMember", ["readonly"]);
          }
        }
      }
    }
  }
  return tokensBuilder.build();
}
