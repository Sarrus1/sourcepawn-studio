import {
  workspace as Workspace,
  TextDocument,
  Position,
  CompletionItemKind,
  Range,
} from "vscode";
import { dirname, join, resolve } from "path";
import { existsSync } from "fs";
import { URI } from "vscode-uri";

import { SPItem, IncludeItem } from "./spItems";
import {
  getLastFuncName,
  isInAComment,
  isFunction,
  getLastEnumStructNameOrMethodMap,
  isInAString,
} from "../Providers/spDefinitionProvider";
import { globalIdentifier } from "../Misc/spConstants";
import { FileItems } from "./spFilesRepository";
import {
  getTypeOfVariable,
  getAllInheritances,
} from "./spItemsPropertyGetters";
import { getAllPossibleIncludeFolderPaths } from "./spFileHandlers";
import { ItemsRepository } from "./spItemsRepository";
import { findMainPath } from "../spUtils";
import { getIncludeExtension } from "./spUtils";

/**
 * Returns an array of all the items parsed from a file and its known includes.
 * @param  {ItemsRepository} itemsRepo      The itemsRepository object constructed in the activation event.
 * @param  {URI} uri                        The URI of the file we are getting the items for.
 * @returns SPItem
 */
export function getAllItems(itemsRepo: ItemsRepository, uri: URI): SPItem[] {
  const mainPath = findMainPath(uri);
  if (mainPath !== undefined) {
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
function getFileItems(uri: string): SPItem[] {
  let items: FileItems = this.fileItems.get(uri);
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

  // First check if we are dealing with a method or property.
  let isMethod = false;
  let isConstructor = false;
  let match: RegExpMatchArray;

  if (isInAComment(range, document.uri, allItems) || isInAString(range, line)) {
    return undefined;
  }

  // Generate an include item if the line is an #include statement and return it.
  let includeItem = makeIncludeItem(document, line, position);
  if (includeItem !== undefined) {
    return includeItem;
  }

  if (range.start.character > 1) {
    let newPosStart = new Position(range.start.line, range.start.character - 2);
    let newPosEnd = new Position(range.start.line, range.start.character);
    let newRange = new Range(newPosStart, newPosEnd);
    let char = document.getText(newRange);
    isMethod = /(?:\w+\.|\:\:)/.test(char);
    if (!isMethod) {
      let newPosStart = new Position(range.start.line, 0);
      let newPosEnd = new Position(range.start.line, range.end.character);
      let newRange = new Range(newPosStart, newPosEnd);
      let line = document.getText(newRange);
      match = line.match(/new\s+(\w+)$/);
      if (match) {
        isConstructor = true;
      }
    }
  }

  let lastFunc: string = getLastFuncName(position, document, allItems);
  let {
    lastEnumStructOrMethodMap,
    isAMethodMap,
  } = getLastEnumStructNameOrMethodMap(position, document, allItems);
  // If we match a property or a method of an enum struct
  // but not a local scopped variable inside an enum struct's method.
  if (
    lastEnumStructOrMethodMap !== globalIdentifier &&
    lastFunc === globalIdentifier &&
    !isAMethodMap
  ) {
    let items = allItems.filter(
      (item) =>
        [
          CompletionItemKind.Method,
          CompletionItemKind.Property,
          CompletionItemKind.Constructor,
        ].includes(item.kind) &&
        item.parent === lastEnumStructOrMethodMap &&
        item.name === word
    );
    if (items.length !== 0) {
      return items;
    }
  }

  if (isMethod) {
    let line = document.lineAt(position.line).text;
    // If we are dealing with a method or property, look for the type of the variable
    let { variableType, words } = getTypeOfVariable(
      line,
      position,
      allItems,
      lastFunc,
      lastEnumStructOrMethodMap
    );
    // Get inheritances from methodmaps
    let variableTypes: string[] = getAllInheritances(variableType, allItems);
    // Find and return the matching item
    let items = allItems.filter(
      (item) =>
        [
          CompletionItemKind.Method,
          CompletionItemKind.Property,
          CompletionItemKind.Constructor,
        ].includes(item.kind) &&
        variableTypes.includes(item.parent) &&
        item.name === word
    );
    return items;
  }

  if (isConstructor) {
    let items = itemsRepo
      .getAllItems(document.uri)
      .filter(
        (item) =>
          item.kind === CompletionItemKind.Constructor && item.name === match[1]
      );
    return items;
  }
  // Check if we are dealing with a function
  let bIsFunction = isFunction(
    range,
    document,
    document.lineAt(position.line).text.length
  );
  let items = [];
  if (bIsFunction) {
    if (lastEnumStructOrMethodMap !== globalIdentifier) {
      // Check for functions and methods
      items = allItems.filter((item) => {
        if (
          [CompletionItemKind.Method, CompletionItemKind.Constructor].includes(
            item.kind
          ) &&
          item.name === word &&
          item.parent === lastEnumStructOrMethodMap
        ) {
          return true;
        } else if (
          [CompletionItemKind.Function, CompletionItemKind.Interface].includes(
            item.kind
          ) &&
          item.name === word
        ) {
          return true;
        }
        return false;
      });
      return items;
    } else {
      items = allItems.filter(
        (item) =>
          [CompletionItemKind.Function, CompletionItemKind.Interface].includes(
            item.kind
          ) && item.name === word
      );
      return items;
    }
  }
  items = allItems.filter(
    (item) =>
      ![
        CompletionItemKind.Method,
        CompletionItemKind.Property,
        CompletionItemKind.Constructor,
        CompletionItemKind.Function,
      ].includes(item.kind) &&
      item.name === word &&
      item.parent === lastFunc
  );
  if (items.length > 0) {
    return items;
  }
  items = allItems.filter((item) => {
    if (
      [
        CompletionItemKind.Method,
        CompletionItemKind.Property,
        CompletionItemKind.Constructor,
      ].includes(item.kind)
    ) {
      return false;
    }
    if (item.parent !== undefined) {
      if (
        [CompletionItemKind.Class, CompletionItemKind.EnumMember].includes(
          item.kind
        )
      ) {
        return item.name === word;
      }
      if (item.enumStructName !== undefined) {
        return (
          item.parent === globalIdentifier &&
          item.name === word &&
          item.enumStructName === lastEnumStructOrMethodMap
        );
      }
      return item.parent === globalIdentifier && item.name === word;
    }
    return item.name === word;
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
    return undefined;
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
}
