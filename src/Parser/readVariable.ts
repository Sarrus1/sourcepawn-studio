import { spParserArgs } from "./interfaces";
import { VariableDeclaration } from "./interfaces";
import { globalItem } from "../Misc/spConstants";
import { parsedLocToRange } from "./utils";
import { addVariableItem } from "./addVariableItem";
import { processDocStringComment } from "./processComment";

export function readVariable(
  parserArgs: spParserArgs,
  content: VariableDeclaration
): void {
  let variableType = "",
    modifier = "",
    processedDeclType = "";
  if (content.variableType) {
    variableType = content.variableType.name.id;
    modifier = content.variableType.modifier || "";
  }
  if (content.accessModifiers != null) {
    processedDeclType = content.accessModifiers.join(" ");
  }
  content.declarations.forEach((e) => {
    const range = parsedLocToRange(e.id.loc, parserArgs);
    const { doc, dep } = processDocStringComment(content.doc);
    const arrayInitialer = e.arrayInitialer || "";
    addVariableItem(
      parserArgs,
      e.id.id,
      variableType,
      range,
      globalItem,
      doc,
      `${processedDeclType}${variableType}${modifier}${
        e.id.id
      }${arrayInitialer.trim()};`.trim(),
      content.accessModifiers
    );
  });
  return;
}
