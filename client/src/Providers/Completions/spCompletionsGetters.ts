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
  getLastFuncName,
  getLastEnumStructNameOrMethodMap,
} from "../../Providers/spDefinitionProvider";
import { SPItem } from "../../Backend/spItems";
import { getAllPossibleIncludeFolderPaths } from "../../Backend/spFileHandlers";
import { ItemsRepository } from "../../Backend/spItemsRepository";
import { isMethodCall } from "../../Backend/spUtils";

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
  const allItems: SPItem[] = itemsRepo.getAllItems(document.uri);
  if (allItems === []) {
    return;
  }

  const line = document.lineAt(position.line).text;
  const isMethod = isMethodCall(line, position);
  const lastFunc: string = getLastFuncName(position, document, allItems);

  let items = new Set<CompletionItem>();

  if (!isMethod) {
    for (let item of allItems) {
      if (
        !(
          item.kind === CompletionItemKind.Method ||
          item.kind === CompletionItemKind.Property
        )
      ) {
        items.add(item.toCompletionItem(document.uri.fsPath, lastFunc));
      }
    }
    // Make sure no undefined objects are present.
    items.delete(undefined);
    return new CompletionList(Array.from(items).filter((e) => e !== undefined));
  }

  const {
    lastEnumStructOrMethodMap,
    isAMethodMap,
  } = getLastEnumStructNameOrMethodMap(position, document, allItems);
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
      items.add(item.toCompletionItem(document.uri.fsPath, lastFunc));
    }
  }
  // Make sure no undefined objects are present.
  items.delete(undefined);
  return new CompletionList(Array.from(items).filter((e) => e !== undefined));
}
