import {
  EnumstructDeclaration,
  EnumstructMemberDeclaration,
  spParserArgs,
} from "./interfaces";
import { parsedLocToRange } from "./utils";
import { EnumStructItem } from "../Backend/Items/spEnumStructItem";
import { processDocStringComment } from "./processComment";
import { readFunctionAndMethod } from "./readFunctionAndMethod";
import { addVariableItem } from "./addVariableItem";

/**
 * Process an enum struct declaration.
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {EnumstructDeclaration} res  Object containing the enum struct declaration details.
 * @returns void
 */
export function readEnumStruct(
  parserArgs: spParserArgs,
  res: EnumstructDeclaration
): void {
  const { doc, dep } = processDocStringComment(res.doc);
  const enumStructItem = new EnumStructItem(
    res.id.id,
    parserArgs.filePath,
    doc,
    parsedLocToRange(res.id.loc, parserArgs),
    parsedLocToRange(res.loc, parserArgs)
  );
  parserArgs.fileItems.items.push(enumStructItem);
  readEnumstructMembers(parserArgs, enumStructItem, res.body);
}

/**
 * Process the body of an enum struct.
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {EnumStructItem} enumstructItem  The parent of the enum struct members.
 * @param  {EnumstructMemberDeclaration[]} body  The body of the enum struct to parse.
 * @returns void
 */
function readEnumstructMembers(
  parserArgs: spParserArgs,
  enumstructItem: EnumStructItem,
  body: EnumstructMemberDeclaration[]
): void {
  if (!body) {
    return;
  }
  body.forEach((e) => {
    switch (e.type) {
      case "MethodDeclaration":
        readFunctionAndMethod(
          parserArgs,
          e.accessModifier,
          e.returnType,
          e.id,
          e.loc,
          undefined,
          e.params,
          e.body,
          e.txt,
          enumstructItem
        );
        break;
      case "VariableDeclaration":
        let variableType = "",
          modifier = "",
          processedDeclType = "";
        if (e.variableType) {
          variableType = e.variableType.name.id;
          modifier = e.variableType.modifier || "";
        }
        if (typeof e.accessModifiers === "string") {
          processedDeclType = e.accessModifiers;
        } else if (Array.isArray(e.accessModifiers)) {
          processedDeclType = e.accessModifiers.join(" ");
        }
        const range = parsedLocToRange(e.declarations[0].id.loc);
        const name = e.declarations[0].id.id;
        addVariableItem(
          parserArgs,
          name,
          variableType,
          range,
          enumstructItem,
          "",
          `${processedDeclType} ${variableType}${modifier}${name};`.trim()
        );
    }
  });
}
