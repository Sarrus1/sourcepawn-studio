import { spParserArgs } from "./spParser";
import { VariableDeclaration } from "./interfaces";
import { globalItem } from "../Misc/spConstants";
import { parsedLocToRange } from "./utils";
import { addVariableItem } from "./addVariableItem";
import { EnumStructItem } from "../Backend/Items/spEnumStructItem";
import { ConstantItem } from "../Backend/Items/spConstantItem";
import { processDocStringComment } from "./processComment";

export function readVariable(
  parserArgs: spParserArgs,
  content: VariableDeclaration,
  parent: EnumStructItem | ConstantItem = globalItem
): void {
  content.declarations.forEach((e) => {
    const range = parsedLocToRange(e.id.loc, parserArgs);
    const { doc, dep } = processDocStringComment(content.doc);

    addVariableItem(
      parserArgs,
      e.id.id,
      content.variableType ? content.variableType.id : "",
      range,
      globalItem,
      doc,
      e.id.id + parent.name
    );
  });
  return;
}
