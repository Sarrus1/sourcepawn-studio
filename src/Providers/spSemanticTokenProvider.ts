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
  const allItems: SPItem[] = itemsRepo.getAllItems(document.uri);

  // Don't run the semanticProvider if a parsing is running on this file.
  let debouncer = itemsRepo.debouncers.get(document.uri.fsPath);
  if (debouncer?.isRunning) {
    return undefined;
  }

  for (const item of allItems) {
    if (item.kind === CompletionItemKind.Variable) {
      if (item.filePath === document.uri.fsPath) {
        tokensBuilder.push(item.range, "variable", ["declaration"]);
      }
      item.references.forEach((ref) => {
        if (ref.uri.fsPath === document.uri.fsPath) {
          tokensBuilder.push(ref.range, "variable", ["modification"]);
        }
      });
    } else if (
      item.kind === CompletionItemKind.Constant &&
      item.references !== undefined
    ) {
      for (const ref of item.references) {
        if (ref.uri.fsPath === document.uri.fsPath) {
          tokensBuilder.push(ref.range, "macro", ["readonly"]);
        }
      }
    } else if (item.kind === CompletionItemKind.EnumMember) {
      for (const ref of item.references) {
        if (ref.uri.fsPath === document.uri.fsPath) {
          tokensBuilder.push(ref.range, "enumMember", ["readonly"]);
        }
      }
    } else if (item.kind === CompletionItemKind.Function) {
      for (const ref of item.references) {
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
      for (const ref of item.references) {
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
      for (const ref of item.references) {
        if (ref.uri.fsPath === document.uri.fsPath) {
          if (item.range.contains(ref.range)) {
            tokensBuilder.push(ref.range, "class", ["declaration"]);
          } else {
            tokensBuilder.push(ref.range, "class");
          }
        }
      }
    } else if (item.kind === CompletionItemKind.Constructor) {
      for (const ref of item.references) {
        if (ref.uri.fsPath === document.uri.fsPath) {
          if (item.range.contains(ref.range)) {
            tokensBuilder.push(ref.range, "class", ["declaration"]);
          } else {
            tokensBuilder.push(
              ref.range,
              "class",
              item.deprecated ? ["deprecated"] : []
            );
          }
        }
      }
    }
  }
  return tokensBuilder.build();
}
