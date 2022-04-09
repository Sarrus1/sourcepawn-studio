import { spParserArgs } from "./spParser";
import {
  ParserLocation,
  PreprocessorStatement,
  ParsedEnumStructMember,
  ParsedID,
} from "./interfaces";
import { parsedLocToRange } from "./utils";
import { EnumStructItem } from "../Backend/Items/spEnumStructItem";
import { processDocStringComment } from "./processComment";
import { readFunctionAndMethod } from "./readFunctionAndMethod";
import { readVariable } from "./readVariable";

/**
 * Callback for a parsed enum struct.
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {ParsedID} id  The id of the enum struct.
 * @param  {ParserLocation} loc  The location of the enum struct.
 * @param  {(string | PreprocessorStatement)[] | undefined} docstring  The doc comment above the enum.
 * @param  {any} body  The body of the enum struct.
 * @returns void
 */
export function readEnumStruct(
  parserArgs: spParserArgs,
  id: ParsedID,
  loc: ParserLocation,
  docstring: (string | PreprocessorStatement)[] | undefined,
  body: any
): void {
  const { doc, dep } = processDocStringComment(docstring);
  const enumStructItem = new EnumStructItem(
    id.id,
    parserArgs.filePath,
    doc,
    parsedLocToRange(id.loc),
    parsedLocToRange(loc)
  );
  parserArgs.fileItems.set(id.id, enumStructItem);
  body["body"].forEach((e: ParsedEnumStructMember) => {
    if (e["type"] === "MethodDeclaration") {
      readFunctionAndMethod(
        parserArgs,
        e.accessModifier,
        e.returnType,
        e.id,
        e.loc,
        [""],
        e.params,
        e.body,
        enumStructItem
      );
    } else if (e["type"] === "VariableDeclaration") {
      readVariable(parserArgs, e, enumStructItem);
    }
  });
}
