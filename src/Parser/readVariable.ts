import { spParserArgs } from "./spParser";
import { VariableItem } from "../Backend/Items/spVariableItem";
import { ParsedID, VariableDeclaration } from "./interfaces";
import { globalIdentifier, globalItem } from "../Misc/spConstants";
import { parsedLocToRange } from "./utils";
import { SPItem } from "../Backend/Items/spItems";

export function readVariable(
  parserArgs: spParserArgs,
  declarationType: string[] | null,
  type: ParsedID | null,
  declarations: VariableDeclaration[] | null,
  docstring: string | null,
  parent: SPItem = globalItem
): void {
  if (declarations === undefined || declarations === null) {
    return;
  }
  declarations.forEach((e) => {
    const range = parsedLocToRange(e.id.loc);
    const variableItem = new VariableItem(
      e.id.id,
      parserArgs.filePath,
      parent,
      range,
      type ? type.id : "",
      globalIdentifier,
      docstring
    );
    parserArgs.fileItems.set(e.id.id, variableItem);
  });
  return;
}
