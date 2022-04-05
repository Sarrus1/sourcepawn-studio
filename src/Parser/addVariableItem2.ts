import { spParserArgs } from "./spParser";
import { VariableItem } from "../Backend/Items/spVariableItem";
import { SPItem } from "../Backend/Items/spItems";
import { Range } from "vscode";
import { parse } from "querystring";

/**
 * Save a variable and generate the appropriate key for the Map it is stored in.
 * The key is a concatenation of the following variables:
 * - *varName*: The name of the variable
 * - *scope*: The scope of the variable (the last function's name or globalIdentifier)
 * - *enumStructName*: The name of the enum struct (empty if none)
 * - *lastFuncName*: The name of the last function (empty if none)
 */
export function addVariableItem(
  parserArgs: spParserArgs,
  name: string,
  type: string,
  range: Range,
  parent: SPItem,
  desc: string,
  key: string
): void {
  const variableItem = new VariableItem(
    name,
    parserArgs.filePath,
    parent,
    range,
    type,
    undefined,
    desc
  );

  parserArgs.fileItems.set(key, variableItem);
}
