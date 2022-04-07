import { spParserArgs } from "./spParser";
import { VariableDeclaration } from "./interfaces";
import { globalItem } from "../Misc/spConstants";
import { parsedLocToRange } from "./utils";
import { addVariableItem } from "./addVariableItem";
import { EnumStructItem } from "../Backend/Items/spEnumStructItem";
import { ConstantItem } from "../Backend/Items/spConstantItem";

export function readVariable(
  parserArgs: spParserArgs,
  content: VariableDeclaration,
  parent: EnumStructItem | ConstantItem = globalItem
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
      e.id.id + parent.name
    );
  });
  return;
}
