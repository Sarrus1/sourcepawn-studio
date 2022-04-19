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
  let variableType = "",
    modifier = "",
    processedDeclType = "";
  if (content.variableType) {
    variableType = content.variableType.name.id;
    modifier = content.variableType.modifier;
  }
  if (typeof content.variableDeclarationType === "string") {
    processedDeclType = content.variableDeclarationType;
  } else if (Array.isArray(content.variableDeclarationType)) {
    processedDeclType = content.variableDeclarationType.join(" ");
  }
  content.declarations.forEach((e) => {
    const range = parsedLocToRange(e.id.loc, parserArgs);
    const { doc, dep } = processDocStringComment(content.doc);
    addVariableItem(
      parserArgs,
      e.id.id,
      variableType,
      range,
      globalItem,
      doc,
      `${processedDeclType} ${variableType}${modifier}${e.id.id};`.trim(),
      `${e.id.id}-${parent.name}`
    );
  });
  return;
}
