import { Range } from "vscode";

import { spParserArgs } from "./interfaces";
import { VariableItem } from "../Backend/Items/spVariableItem";
import { SPItem } from "../Backend/Items/spItems";

/**
 * TODO: Remove this function.
 * Save a variable item.
 */
export function addVariableItem(
  parserArgs: spParserArgs,
  name: string,
  type: string,
  range: Range,
  parent: SPItem,
  docstring: string,
  details: string,
  accessModifiers?: string[] | undefined
): void {
  const variableItem = new VariableItem(
    name,
    parserArgs.filePath,
    parent,
    range,
    type,
    details,
    docstring,
    accessModifiers
  );

  parserArgs.fileItems.items.push(variableItem);
}
