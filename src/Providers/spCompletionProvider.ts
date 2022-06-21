import {
  TextDocument,
  Position,
  CancellationToken,
  CompletionList,
  CompletionItemKind,
  Range,
} from "vscode";

import { getTypeOfVariable } from "../Backend/spItemsPropertyGetters";
import { ItemsRepository } from "../Backend/spItemsRepository";
import { getLastFunc, getLastESOrMM } from "./spDefinitionProvider";
import {
  getCompletionListFromPosition,
  getIncludeFileCompletionList,
} from "./Completions/spCompletionsGetters";
import { TypedefItem } from "../Backend/Items/spTypedefItem";
import { TypesetItem } from "../Backend/Items/spTypesetItem";
import { FunctionItem } from "../Backend/Items/spFunctionItem";

export async function completionProvider(
  itemsRepo: ItemsRepository,
  document: TextDocument,
  position: Position,
  token: CancellationToken
): Promise<CompletionList> {
  const text = document
    .lineAt(position.line)
    .text.substring(0, position.character);

  // If the trigger char is a space, check if there is a
  // "new" behind, and deal with the associated constructor.
  if (text[text.length - 1] === " ") {
    const allItems = itemsRepo.getAllItems(document.uri);
    if (position.character > 0) {
      const line = document
        .lineAt(position.line)
        .text.substring(0, position.character);

      const match = line.match(
        /(\w*)\s+([\w.\(\)]+)(?:\[[\w+ \d]+\])*\s*\=\s*new\s+(\w*)$/
      );
      if (match) {
        let type: string | undefined;

        if (!match[1]) {
          // If the variable is not declared here, look up its type, as it
          // has not yet been parsed.
          const lastFunc = getLastFunc(position, document, allItems);
          const newPos = new Position(1, match[2].length + 1);
          const lastEnumStructOrMethodMap = getLastESOrMM(
            position,
            document.uri.fsPath,
            allItems
          );
          const { variableType, words: _ } = getTypeOfVariable(
            // Hack to use getTypeOfVariable
            match[2] + ".",
            newPos,
            allItems,
            lastFunc,
            lastEnumStructOrMethodMap
          );
          type = variableType;
        } else {
          // If the variable is declared here, search its type directly.
          type = allItems.find(
            (item) =>
              item.kind === CompletionItemKind.Class && item.name === match[1]
          ).name;
        }

        // Filter the item to only keep the constructors.
        const items = allItems.filter(
          (item) => item.kind === CompletionItemKind.Constructor
        );
        return new CompletionList(
          items.map((e) => {
            // Show the associated type's constructor first.
            if (e.name === type) {
              const tmp = e.toCompletionItem();
              tmp.preselect = true;
              return tmp;
            }
            return e.toCompletionItem();
          })
        );
      }
    }
    return new CompletionList();
  }

  if (text[text.length - 1] === "$") {
    const allItems = itemsRepo.getAllItems(document.uri);
    const completions = [];
    const range = new Range(
      position.line,
      position.character - 1,
      position.line,
      position.character + 1
    );

    const TyFu = [
      CompletionItemKind.TypeParameter,
      CompletionItemKind.Function,
    ];

    allItems.forEach((e) => {
      if (!TyFu.includes(e.kind)) {
        return;
      }
      const item = e as TypedefItem | TypesetItem | FunctionItem;
      const completion = item.toSnippet(range);
      if (completion === undefined) {
        return;
      }
      if (Array.isArray(completion)) {
        for (const comp of completion) {
          completions.push(comp);
        }
      } else {
        completions.push(completion);
      }
    });
    return new CompletionList(completions);
  }

  // Check if we are dealing with an include.
  let match = text.match(/^\s*#\s*include\s*(?:\<([^>]*)\>?)/);
  let useAp = false;
  if (!match) {
    match = text.match(/^\s*#\s*include\s*(?:\"([^\"]*)\"?)/);
    useAp = true;
  }
  if (match) {
    return getIncludeFileCompletionList(
      itemsRepo.documents,
      document,
      match[1],
      useAp
    );
  }
  match = text.match(
    /^\s*(?:HookEvent|HookEventEx)\s*\(\s*(\"[^\"]*|\'[^\']*)$/
  );
  if (match) {
    return itemsRepo.getEventCompletions();
  }
  if (['"', "'", "<", "/", "\\"].includes(text[text.length - 1]))
    return undefined;
  if (/[^:]\:$/.test(text)) {
    return undefined;
  }
  return getCompletionListFromPosition(itemsRepo, document, position);
}
