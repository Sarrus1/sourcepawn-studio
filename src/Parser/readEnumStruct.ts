import { spParserArgs } from "./interfaces";
import {
  ParserLocation,
  ParsedEnumStructMember,
  ParsedID,
  ParsedComment,
} from "./interfaces";
import { parsedLocToRange } from "./utils";
import { EnumStructItem } from "../Backend/Items/spEnumStructItem";
import { processDocStringComment } from "./processComment";
import { readFunctionAndMethod } from "./readFunctionAndMethod";
import { addVariableItem } from "./addVariableItem";
import { globalIdentifier } from "../Misc/spConstants";

/**
 * Callback for a parsed enum struct.
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {ParsedID} id  The id of the enum struct.
 * @param  {ParserLocation} loc  The location of the enum struct.
 * @param  {ParsedComment} docstring  The doc comment above the enum.
 * @param  {any} body  The body of the enum struct.
 * @returns void
 */
export function readEnumStruct(
  parserArgs: spParserArgs,
  id: ParsedID,
  loc: ParserLocation,
  docstring: ParsedComment,
  body: any
): void {
  const { doc, dep } = processDocStringComment(docstring);
  const enumStructItem = new EnumStructItem(
    id.id,
    parserArgs.filePath,
    doc,
    parsedLocToRange(id.loc, parserArgs),
    parsedLocToRange(loc, parserArgs)
  );
  parserArgs.fileItems.items.push(enumStructItem);
  body["body"].forEach((e: ParsedEnumStructMember) => {
    if (e["type"] === "MethodDeclaration") {
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
        enumStructItem
      );
    } else if (e["type"] === "VariableDeclaration") {
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
        enumStructItem,
        "",
        `${processedDeclType} ${variableType}${modifier}${name};`.trim()
      );
    }
  });
}
