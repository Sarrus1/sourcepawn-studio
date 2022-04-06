import { spParserArgs } from "./spParser";
import { VariableItem } from "../Backend/Items/spVariableItem";
import { VariableDeclaration } from "./interfaces";
import { globalIdentifier, globalItem } from "../Misc/spConstants";
import { parsedLocToRange } from "./utils";
import { addVariableItem } from "./addVariableItem2";

export function readVariable(
  parserArgs: spParserArgs,
  content: VariableDeclaration
): void {
  content.declarations.forEach((e) => {
    const range = parsedLocToRange(e.id.loc);
    addVariableItem(
      parserArgs,
      e.id.id,
      content.variableType ? content.variableType.id : "",
      range,
      globalItem,
      content.doc,
      e.id.id + globalIdentifier
    );
  });
  return;
}
