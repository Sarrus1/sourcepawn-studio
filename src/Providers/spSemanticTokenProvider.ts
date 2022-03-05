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
    if (
      item.kind === CompletionItemKind.Constant &&
      item.references !== undefined
    ) {
      for (let ref of item.references) {
        if (ref.uri.fsPath === document.uri.fsPath) {
          tokensBuilder.push(ref.range, "variable", ["readonly"]);
        }
      }
    } else if (item.kind === CompletionItemKind.EnumMember) {
      for (let ref of item.references) {
        if (ref.uri.fsPath === document.uri.fsPath) {
          tokensBuilder.push(ref.range, "enumMember", ["readonly"]);
        }
      }
    } else if (item.kind === CompletionItemKind.Function) {
      for (let ref of item.references) {
        if (ref.uri.fsPath === document.uri.fsPath) {
          if (item.range.contains(ref.range)) {
            tokensBuilder.push(ref.range, "function", ["declaration"]);
          } else {
            tokensBuilder.push(
              ref.range,
              "function",
              item.deprecated ? ["deprecated"] : []
            );
          }
        }
      }
    } else if (item.kind === CompletionItemKind.Method) {
      for (let ref of item.references) {
        if (ref.uri.fsPath === document.uri.fsPath) {
          if (item.range.contains(ref.range)) {
            tokensBuilder.push(ref.range, "method", ["declaration"]);
          } else {
            tokensBuilder.push(
              ref.range,
              "method",
              item.deprecated ? ["deprecated"] : []
            );
          }
        }
      }
    } else if (item.kind === CompletionItemKind.Class) {
      for (let ref of item.references) {
        if (ref.uri.fsPath === document.uri.fsPath) {
          if (item.range.contains(ref.range)) {
            tokensBuilder.push(ref.range, "class", ["declaration"]);
          } else {
            tokensBuilder.push(ref.range, "class");
          }
        }
      }
    }
  }
  return tokensBuilder.build();
}
