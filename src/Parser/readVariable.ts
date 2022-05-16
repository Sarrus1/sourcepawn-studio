import { spParserArgs } from "./interfaces";
import { VariableDeclaration } from "./interfaces";
import { globalItem } from "../Misc/spConstants";
import { parsedLocToRange } from "./utils";
import { processDocStringComment } from "./processComment";
import { VariableItem } from "../Backend/Items/spVariableItem";

/**
 * Process a global variable declaration.
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {MethodmapDeclaration} res  The object containing the variable declaration details.
 * @returns void
 */
export function readVariable(
  parserArgs: spParserArgs,
  res: VariableDeclaration
): void {
  let variableType = "",
    modifier = "",
    processedDeclType = "";
  if (res.variableType) {
    variableType = res.variableType.name.id;
    modifier = res.variableType.modifier || "";
  }
  if (res.accessModifiers != null) {
    processedDeclType = res.accessModifiers.join(" ");
  }
  res.declarations.forEach((e) => {
    const range = parsedLocToRange(e.id.loc, parserArgs);
    const { doc, dep } = processDocStringComment(res.doc);
    const arrayInitialer = e.arrayInitialer || "";
    const variableItem = new VariableItem(
      e.id.id,
      parserArgs.filePath,
      globalItem,
      range,
      variableType,
      `${processedDeclType}${variableType}${modifier}${
        e.id.id
      }${arrayInitialer.trim()};`.trim(),
      doc,
      res.accessModifiers
    );
    parserArgs.fileItems.items.push(variableItem);
  });
}
