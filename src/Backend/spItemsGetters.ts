import { TextDocument, Position, CompletionItemKind, Range } from "vscode";
import { resolve } from "path";
import { existsSync } from "fs";
import { URI } from "vscode-uri";

import { SPItem } from "./Items/spItems";
import { IncludeItem } from "./Items/spIncludeItem";
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

const FI = [CompletionItemKind.Function, CompletionItemKind.Interface];

const MC = [CompletionItemKind.Method, CompletionItemKind.Constructor];

const MPC = MC.concat([CompletionItemKind.Property]);

const MPCF = MPC.concat([CompletionItemKind.Function]);

const CE = [CompletionItemKind.Class, CompletionItemKind.EnumMember];

enum ObjectType {
  Variable,
  Method,
  Constructor,
  Function,
}

/**
 * Returns an array of all the items parsed from a file and its known includes.
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

  let type = getType(range, document, position);

  const lastFunc: string = getLastFuncName(position, document, allItems);

  const {
    lastEnumStructOrMethodMap,
    isAMethodMap,
  } = getLastEnumStructNameOrMethodMap(position, document, allItems);

  // If we match a property or a method of an enum struct
  // but not a local scopped variable inside an enum struct's method.
  let items = makeEnumStructMethodItem(
    lastEnumStructOrMethodMap,
    lastFunc,
    isAMethodMap,
    allItems,
    word
  );
  if (items.length > 0) {
    return items;
  }

  if (type === ObjectType.Method) {
    // If we are dealing with a method or property, look for the type of the variable
    const { variableType, words } = getTypeOfVariable(
      line,
      position,
      allItems,
      lastFunc,
      lastEnumStructOrMethodMap
    );
    const variableTypes = getAllInheritances(variableType, allItems);
    return allItems.filter(
      (item) =>
        MPC.includes(item.kind) &&
        variableTypes.includes(item.parent) &&
        item.name === word
    );
  }

  if (type === ObjectType.Constructor) {
    const match = line.match(/new\s+(\w+)/);
    if (match) {
      return allItems.filter(
        (item) =>
          item.kind === CompletionItemKind.Constructor && item.name === match[1]
      );
    }
  }

  items = [];

  if (type === ObjectType.Function) {
    return makeFunctionOrMethodItem(word, lastEnumStructOrMethodMap, allItems);
  }

  items = allItems.filter(
    (item) =>
      !MPCF.includes(item.kind) &&
      item.name === word &&
      item.parent === lastFunc
  );
  if (items !== undefined && items.length > 0) {
    return items;
  }

  return allItems.filter(lastResortItemFilterCallback, {
    lastEnumStructOrMethodMap,
    word,
  });
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

/**
 * Try to find a corresponding EnumStructMember item from a name, and the file scope.
 * @param  {string} lastEnumStructOrMethodMap
 * @param  {string} lastFunc
 * @param  {boolean} isAMethodMap
 * @param  {SPItem[]} allItems
 * @param  {string} name
 * @returns SPItem
 */
function makeEnumStructMethodItem(
  lastEnumStructOrMethodMap: string,
  lastFunc: string,
  isAMethodMap: boolean,
  allItems: SPItem[],
  name: string
): SPItem[] {
  if (
    lastEnumStructOrMethodMap !== globalIdentifier &&
    lastFunc === globalIdentifier &&
    !isAMethodMap
  ) {
    let items = allItems.filter(
      (item) =>
        MPC.includes(item.kind) &&
        item.parent === lastEnumStructOrMethodMap &&
        item.name === name
    );
    if (items !== undefined && items.length > 0) {
      return items;
    }
  }
  return [];
}

/**
 * Try to find a corresponding function or method to a word and a scope.
 * @param  {string} name
 * @param  {string} lastEnumStructOrMethodMap
 * @param  {SPItem[]} allItems
 * @returns SPItem
 */
function makeFunctionOrMethodItem(
  name: string,
  lastEnumStructOrMethodMap: string,
  allItems: SPItem[]
): SPItem[] {
  const items = allItems.filter(
    (item) => FI.includes(item.kind) && item.name === name
  );
  if (lastEnumStructOrMethodMap === globalIdentifier) {
    return items;
  }
  return allItems
    .filter(
      (item) =>
        MC.includes(item.kind) &&
        item.name === name &&
        item.parent === lastEnumStructOrMethodMap
    )
    .concat(items);
}

/**
 * Checks if we are dealing with a method, a constructor, a function, or a regular variable.
 * @param  {Range} range            The range to check.
 * @param  {TextDocument} document  The document corresponding to the range.
 * @param  {Position} position      The position of the line to check.
 * @returns ObjectType
 */
function getType(
  range: Range,
  document: TextDocument,
  position: Position
): ObjectType {
  // Check if we are dealing with a define declaration with opening (.
  if (
    /^\s*#define\s+(\w+)\s+(?:(.+)?(?=(?:\/\*|$|\/\/)))?/.test(
      document.lineAt(position.line).text
    )
  ) {
    return ObjectType.Variable;
  }
  if (range.start.character <= 1) {
    if (
      isFunction(range, document, document.lineAt(position.line).text.length)
    ) {
      return ObjectType.Function;
    }
    return ObjectType.Variable;
  }
  let newRange = new Range(
    range.start.line,
    range.start.character - 2,
    range.start.line,
    range.start.character
  );
  let char = document.getText(newRange);
  if (/(?:\w+\.|\:\:)/.test(char)) {
    return ObjectType.Method;
  }
  newRange = new Range(
    range.start.line,
    0,
    range.start.line,
    range.end.character
  );
  char = document.getText(newRange);
  if (/new\s+(\w+)$/.test(char)) {
    return ObjectType.Constructor;
  }
  if (isFunction(range, document, document.lineAt(position.line).text.length)) {
    return ObjectType.Function;
  }
  return ObjectType.Variable;
}

/**
 * Callback of a filter function to try and find a corresponding object.
 * @param  {SPItem} item
 * @returns boolean
 */
function lastResortItemFilterCallback(item: SPItem): boolean {
  if (MPC.includes(item.kind)) {
    return false;
  }
  if (item.parent !== undefined) {
    if (CE.includes(item.kind)) {
      return item.name === this.word;
    }
    if (item.enumStructName !== undefined) {
      return (
        item.parent === globalIdentifier &&
        item.name === this.word &&
        item.enumStructName === this.lastEnumStructOrMethodMap
      );
    }
    return item.parent === globalIdentifier && item.name === this.word;
  }
  return item.name === this.word;
}
