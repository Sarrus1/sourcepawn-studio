import { Position, CompletionItemKind } from "vscode";
import { SPItem } from "./spItems";
import { globalIdentifier } from "../Misc/spConstants";

export interface VariableType {
  variableType: string;
  words: string[];
}

export interface ParsedLine {
  words: string[];
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
 * @param  {string} lastFuncName
 * @param  {string} lastEnumStructOrMethodMap
 * @returns VariableType
 */
export function getTypeOfVariable(
  line: string,
  position: Position,
  allItems: SPItem[],
  lastFuncName: string,
  lastEnumStructOrMethodMap: string
): VariableType {
  let { words, isNameSpace } = parseMethodsFromLine(line, position);
  let variableType: string;

  if (isNameSpace) {
    variableType = words[words.length - 1];
  } else {
    if (
      lastEnumStructOrMethodMap !== globalIdentifier &&
      words[words.length - 1] === "this"
    ) {
      variableType = lastEnumStructOrMethodMap;
    } else {
      variableType = allItems.find(
        (e) =>
          (e.kind === CompletionItemKind.Variable &&
            [globalIdentifier, lastFuncName].includes(e.parent) &&
            e.name === words[words.length - 1]) ||
          (e.kind === CompletionItemKind.Function &&
            e.name === words[words.length - 1]) ||
          (e.kind === CompletionItemKind.Class &&
            e.name === words[words.length - 1])
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
          e.parent === variableType &&
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
 * @param  {string} line
 * @param  {Position} position
 * @returns ParsedLine
 */
function parseMethodsFromLine(line: string, position: Position): ParsedLine {
  let i = position.character - 1;
  let bCounter = 0;
  let pCounter = 0;
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
  let wordCounter = 0;
  let words: string[] = [""];
  while (i >= 0) {
    if (line[i] === "]") {
      bCounter++;
      i--;
      continue;
    }
    if (line[i] === "[") {
      bCounter--;
      i--;
      continue;
    }
    if (line[i] === ")") {
      pCounter++;
      i--;
      continue;
    }
    if (line[i] === "(") {
      pCounter--;
      i--;
      continue;
    }
    if (bCounter === 0 && pCounter === 0) {
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
