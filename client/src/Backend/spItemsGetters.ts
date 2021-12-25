import {
  CompletionItem,
  TextDocument,
  Position,
  CompletionList,
  CompletionItemKind,
} from "vscode";
import { basename } from "path";
import { URI } from "vscode-uri";

import {
  GetLastFuncName,
  getLastEnumStructNameOrMethodMap,
} from "../Providers/spDefinitionProvider";
import { SPItem } from "./spItems";
import { getAllPossibleIncludeFolderPaths } from "./spFileHandlers";
import { ItemsRepository } from "./spItemsRepository";
import { isMethodCall } from "./spUtils";

/**
 * Generate a CompletionList object of the possible includes file that can fit the already typed #include statement.
 * @param  {Set<string>} knownIncs    Set of parsed include files (.sp and .inc).
 * @param  {TextDocument} document    The document being edited.
 * @param  {string} tempName          The string that has already been typed in the #include statement.
 * @returns CompletionList
 */
export function getIncludeFileCompletionList(
  knownIncs: Set<string>,
  document: TextDocument,
  tempName: string
): CompletionList {
  const isQuoteInclude: boolean = tempName.includes('"');
  const incURIs = getAllPossibleIncludeFolderPaths(document.uri).map((e) =>
    URI.file(e)
  );
  const prevPath = tempName.replace(/((?:[^\'\<\/]+\/)+)+/, "$1");

  let items: CompletionItem[] = [];

  Array.from(knownIncs).forEach((e) =>
    incURIs.find((incURI) => {
      const fileMatchRe = RegExp(
        `${incURI.toString()}\\/${prevPath}[^<>:;,?"*|/]+\\.(?:inc|sp)$`
      );
      if (fileMatchRe.test(e)) {
        const path = URI.parse(e).fsPath;
        items.push({
          label: basename(path),
          kind: CompletionItemKind.File,
          detail: path,
        });
        return true;
      }
    })
  );

  const availableIncFolderPaths = new Set<string>();
  knownIncs.forEach((e) => {
    incURIs.forEach((incURI) => {
      const folderMatchRe = RegExp(
        `${incURI.toString()}\\/${prevPath}(\\w[^*/><?\\|:]+)\\/`
      );
      const match = e.match(folderMatchRe);
      if (match) {
        availableIncFolderPaths.add(`${incURI.toString()}/${match[1]}`);
      }
    });
  });

  availableIncFolderPaths.forEach((e) => {
    const path = URI.parse(e).fsPath;
    items.push({
      label: basename(path),
      kind: CompletionItemKind.Folder,
      detail: path,
    });
  });

  return new CompletionList(items);
}

/**
 * Returns a CompletionList object of all the objects available at that position's scope.
 * @param  {ItemsRepository} itemsRepo    The itemsRepository object constructed in the activation event.
 * @param  {TextDocument} document        The document the completions are requested for.
 * @param  {Position} position            The position at which the completions are requested.
 * @returns CompletionList
 */
export function getCompletionListFromPosition(
  itemsRepo: ItemsRepository,
  document: TextDocument,
  position: Position
): CompletionList {
  let line = document.lineAt(position.line).text;
  let isMethod = isMethodCall(line, position);
  let allItems: SPItem[] = itemsRepo.getAllItems(document.uri);
  let completionsList: CompletionList = new CompletionList();
  if (allItems !== []) {
    let lastFunc: string = GetLastFuncName(position, document, allItems);
    let {
      lastEnumStructOrMethodMap,
      isAMethodMap,
    } = getLastEnumStructNameOrMethodMap(position, document, allItems);
    if (isMethod) {
      let { variableType, words } = itemsRepo.getTypeOfVariable(
        line,
        position,
        allItems,
        lastFunc,
        lastEnumStructOrMethodMap
      );
      let variableTypes: string[] = itemsRepo.getAllInheritances(
        variableType,
        allItems
      );
      let existingNames: string[] = [];

      // Prepare check for static methods
      let isMethodMap: boolean;
      if (words.length === 1) {
        let methodmap = allItems.find(
          (e) => e.name === words[0] && e.kind === CompletionItemKind.Class
        );
        isMethodMap = methodmap !== undefined;
      }
      for (let item of allItems) {
        if (
          (item.kind === CompletionItemKind.Method ||
            item.kind === CompletionItemKind.Property) &&
          variableTypes.includes(item.parent) &&
          // Don't include the constructor of the methodmap
          !variableTypes.includes(item.name) &&
          // Check for static methods
          ((!isMethodMap && !item.detail.includes("static")) ||
            (isMethodMap && item.detail.includes("static")))
        ) {
          if (!existingNames.includes(item.name)) {
            completionsList.items.push(
              item.toCompletionItem(document.uri.fsPath, lastFunc)
            );
            existingNames.push(item.name);
          }
        }
      }
      return completionsList;
    }
    let existingNames: string[] = [];
    for (let item of allItems) {
      if (
        !(
          item.kind === CompletionItemKind.Method ||
          item.kind === CompletionItemKind.Property
        )
      ) {
        if (!existingNames.includes(item.name)) {
          // Make sure we don't add a variable to existingNames if it's not in the scope of the current function.
          let newItem = item.toCompletionItem(document.uri.fsPath, lastFunc);
          if (newItem !== undefined) {
            completionsList.items.push(newItem);
            existingNames.push(item.name);
          }
        }
      }
    }
    return completionsList;
  }
}
