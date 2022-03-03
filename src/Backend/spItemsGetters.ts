import { TextDocument, Position, CompletionItemKind, Range } from "vscode";
import { resolve } from "path";
import { existsSync } from "fs";
import { URI } from "vscode-uri";

import { SPItem } from "./Items/spItems";
import { IncludeItem } from "./Items/spIncludeItem";
import {
  getLastEnumStructNameOrMethodMap,
  isInAComment,
  isInAString,
} from "../Providers/spDefinitionProvider";
import { FileItems } from "./spFilesRepository";
import { getAllPossibleIncludeFolderPaths } from "./spFileHandlers";
import { ItemsRepository } from "./spItemsRepository";
import { findMainPath } from "../spUtils";
import { getIncludeExtension } from "./spUtils";
import { globalIdentifier } from "../Misc/spConstants";

/**
 * Returns an array of all the items parsed from a file and its known includes
 * @param  {ItemsRepository} itemsRepo      The itemsRepository object constructed in the activation event.
 * @param  {URI} uri                        The URI of the file we are getting the items for.
 * @returns SPItem
 */
export function getAllItems(itemsRepo: ItemsRepository, uri: URI): SPItem[] {
  const mainPath = findMainPath(uri);
  if (mainPath !== undefined && mainPath !== "") {
    uri = URI.file(mainPath);
  }

  let includes = new Set<string>([uri.toString()]);
  let fileItems = itemsRepo.fileItems.get(uri.toString());
  if (fileItems === undefined) {
    return [];
  }

  getIncludedFiles(itemsRepo, fileItems, includes);
  return Array.from(includes).map(getFileItems, itemsRepo).flat();
}

/**
 * Callback used by the map function in getAllItems. Gets all the items from a parsed file, without its includes.
 * @param  {string} uri    The URI of the file we should get the items for.
 * @returns SPItem
 */
function getFileItems(this: ItemsRepository, uri: string): SPItem[] {
  let items = this.fileItems.get(uri);
  return items !== undefined ? Array.from(items.values()) : [];
}

/**
 * Recursively get all the includes from a FileItems object.
 * @param  {FileItems} fileItems    The object to get the includes from.
 * @param  {Set<string>} includes   The Set to add the include to.
 * @returns void
 */
function getIncludedFiles(
  itemsRepo: ItemsRepository,
  fileItems: FileItems,
  includes: Set<string>
): void {
  for (let include of fileItems.includes) {
    if (includes.has(include.uri)) {
      continue;
    }
    includes.add(include.uri);
    let includeFileItems = itemsRepo.fileItems.get(include.uri);
    if (includeFileItems) {
      getIncludedFiles(itemsRepo, includeFileItems, includes);
    }
  }
}

/**
 * Get corresponding items from a position in a document.
 * @param  {ItemsRepository} itemsRepo      The itemsRepository object constructed in the activation event.
 * @param  {TextDocument} document          The document to get the item from.
 * @param  {Position} position              The position at which to get the item.
 * @returns SPItem
 */
export function getItemFromPosition(
  itemsRepo: ItemsRepository,
  document: TextDocument,
  position: Position
): SPItem[] {
  const range = document.getWordRangeAtPosition(position);
  const allItems = itemsRepo.getAllItems(document.uri);

  const word = document.getText(range);
  const line = document.lineAt(position.line).text;
  if (
    range === undefined ||
    isInAComment(range, document.uri, allItems) ||
    isInAString(range, line)
  ) {
    return [];
  }

  // Generate an include item if the line is an #include statement and return it.
  let includeItem = makeIncludeItem(document, line, position);
  if (includeItem.length > 0) {
    return includeItem;
  }

  let lastEnumStructOrMethodMap = getLastEnumStructNameOrMethodMap(
    position,
    document.uri.fsPath,
    allItems
  );

  let items = allItems.filter((e) => {
    if (e.name !== word) {
      return false;
    }

    if (
      e.kind === CompletionItemKind.Variable &&
      e.parent !== globalIdentifier &&
      allItems.find((e1) => {
        let check =
          [CompletionItemKind.Function, CompletionItemKind.Method].includes(
            e1.kind
          ) &&
          e1.name === e.parent &&
          e1.fullRange.contains(position) &&
          e1.filePath === document.uri.fsPath;

        // Handle variables inside of methods.
        if (lastEnumStructOrMethodMap !== undefined && check) {
          return (
            e.enumStructName === lastEnumStructOrMethodMap.name &&
            lastEnumStructOrMethodMap.fullRange.contains(e1.fullRange) &&
            lastEnumStructOrMethodMap.fullRange.contains(e.range)
          );
        }
        return check;
      })
    ) {
      return true;
    }
    if (e.range !== undefined && range.isEqual(e.range)) {
      return true;
    }
    if (e.references !== undefined) {
      return e.references.find((e) => range.isEqual(e.range)) !== undefined;
    }
    return false;
  });
  return items;
}

/**
 * Try to generate an IncludeItem from an #include statement line and return it.
 * @param  {TextDocument} document          The document the item is generated for.
 * @param  {string} line                    The line to parse.
 * @param  {Position} position              The position to parse.
 * @returns SPItem
 */
function makeIncludeItem(
  document: TextDocument,
  line: string,
  position: Position
): SPItem[] {
  const match =
    line.match(/^\s*#include\s+<([A-Za-z0-9\-_\/.]+)>/) ||
    line.match(/^\s*#include\s+"([A-Za-z0-9\-_\/.]+)"/);
  if (!match) {
    return [];
  }
  let file = match[1];
  const fileStartPos = line.search(file);
  file = getIncludeExtension(file);
  const defRange = new Range(
    position.line,
    fileStartPos,
    position.line,
    fileStartPos + file.length
  );
  const potentialIncludePaths = getAllPossibleIncludeFolderPaths(document.uri);

  const incFolderPath = potentialIncludePaths.find((e) =>
    existsSync(resolve(e, file))
  );
  if (incFolderPath) {
    return [
      new IncludeItem(
        URI.file(resolve(incFolderPath, file)).toString(),
        defRange
      ),
    ];
  }
  return [];
}
