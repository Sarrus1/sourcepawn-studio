import { Position, CompletionItemKind } from "vscode";

import { SPItem } from "./Items/spItems";
import { globalIdentifier, globalItem } from "../Misc/spConstants";
import { FunctionItem } from "./Items/spFunctionItem";
import { MethodItem } from "./Items/spMethodItem";
import { MethodMapItem } from "./Items/spMethodmapItem";

export interface VariableType {
  variableType: string;
  words: string[];
}

export interface ParsedLine {
  words: string[];
  isNameSpace: boolean;
}

export interface MethodIndex {
  i: number;
  isNameSpace: boolean;
}

/**
 * Parses the type of a variable from a line an a position in a document.
 * Returns a VariableType object with the type of the variable as a string, and an array of strings,
 * containing all the words prior to that variable.
 * For example, "foo.bar.baz;" will yield :
 * {
 *   type: "ConVar", // if foo is a ConVar
 *   words: ["foo", "bar"]
 * }
 * @param  {string} line
 * @param  {Position} position
 * @param  {SPItem[]} allItems
 * @param  {string} lastFunc
 * @param  {string} lastEnumStructOrMethodMap
 * @returns VariableType
 */
export function getTypeOfVariable(
  line: string,
  position: Position,
  allItems: SPItem[],
  lastFunc: FunctionItem | MethodItem | undefined,
  lastEnumStructOrMethodMap: SPItem | undefined
): VariableType {
  let { words, isNameSpace } = parseMethodsFromLine(line, position.character);
  let variableType: string;

  const lastFuncName =
    lastFunc === undefined
      ? [globalIdentifier]
      : [globalIdentifier, lastFunc.name];

  if (isNameSpace) {
    variableType = words[words.length - 1];
  } else {
    if (
      lastEnumStructOrMethodMap !== undefined &&
      lastEnumStructOrMethodMap.parent !== globalItem &&
      words[words.length - 1] === "this"
    ) {
      variableType = lastEnumStructOrMethodMap.name;
    } else {
      const enumMemberItem = allItems.find(
        (e) => e.kind === CompletionItemKind.EnumMember && e.name === words[0]
      );
      variableType = allItems.find(
        (e) =>
          (e.kind === CompletionItemKind.Variable &&
            lastFuncName.includes(e.parent.name) &&
            e.name === words[words.length - 1]) ||
          ([CompletionItemKind.Function, CompletionItemKind.Class].includes(
            e.kind
          ) &&
            e.name === words[words.length - 1]) ||
          (e.kind === CompletionItemKind.Class &&
            e.name === words[words.length - 1]) ||
          (enumMemberItem !== undefined &&
            e.kind === CompletionItemKind.Class &&
            (e.name === words[words.length - 1] || e === enumMemberItem.parent))
      ).type;
    }
  }

  if (words.length > 1) {
    words = words.slice(0, words.length - 1).reverse();
    for (let word of words) {
      variableType = allItems.find(
        (e) =>
          (e.kind === CompletionItemKind.Method ||
            e.kind === CompletionItemKind.Property) &&
          e.parent.name === variableType &&
          e.name === word
      ).type;
    }
  }
  return { variableType, words };
}

/**
 * Parses a line and separates the variable and its methods.
 * For example, "foo.bar.baz;" will yield :
 * {
 *   words: ["foo", "bar"],
 *   isNameSpace: false
 * }
 * @param  {string} line        The line being parsed.
 * @param  {number} index       The index at which the parsing should begin.
 * @returns ParsedLine
 */
export function parseMethodsFromLine(line: string, index: number): ParsedLine {
  let { i, isNameSpace } = getMethodIndex(index - 1, line);
  let bCounter = 0;
  let pCounter = 0;
  let wordCounter = 0;
  let words = [""];
  while (i >= 0) {
    if (line[i] === "]") {
      bCounter++;
    } else if (line[i] === "[") {
      bCounter--;
    } else if (line[i] === ")") {
      pCounter++;
    } else if (line[i] === "(") {
      pCounter--;
    } else if (bCounter === 0 && pCounter === 0) {
      if (/\w/.test(line[i])) {
        words[wordCounter] = line[i] + words[wordCounter];
      } else if (line[i] === ".") {
        wordCounter++;
        words[wordCounter] = "";
      } else if (line[i] === ":") {
        i--;
        if (line[i] === ":") {
          wordCounter++;
          words[wordCounter] = "";
          isNameSpace = true;
        }
      } else {
        break;
      }
    }
    i--;
  }
  return { words, isNameSpace };
}

function getMethodIndex(i: number, line: string): MethodIndex {
  let isNameSpace = false;
  while (i >= 0) {
    if (/\w/.test(line[i])) {
      i--;
    } else if (line[i] === ".") {
      i--;
      break;
    } else if (line[i] === ":") {
      i--;
      if (line[i] === ":") {
        i--;
        isNameSpace = true;
        break;
      }
    }
  }
  return { i, isNameSpace };
}

/**
 * Return all the methodmaps which a given methodmap inherits from.
 * @param  {string} methodmap   The name of the methodmap to search inheritances for.
 * @param  {SPItem[]} allItems  All the items known to the document.
 * @returns string
 */
export function getAllInheritances(
  methodmap: MethodMapItem,
  allItems: SPItem[]
): MethodMapItem[] {
  if (methodmap === globalItem || methodmap.parent === globalItem) {
    return [methodmap];
  }
  return [methodmap].concat(
    getAllInheritances(methodmap.parent as MethodMapItem, allItems)
  );
}
