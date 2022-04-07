import { spParserArgs } from "./spParser";
import {
  ParserLocation,
  ParsedEnumMember,
  PreprocessorStatement,
  ParsedEnumStructMember,
} from "./interfaces";
import { parsedLocToRange } from "./utils";
import { EnumStructItem } from "../Backend/Items/spEnumStructItem";
import { processDocStringComment } from "./processComment";
import { readFunctionAndMethod } from "./readFunctionAndMethod";
import { readVariable } from "./readVariable";

/**
 * Callback for a parsed enum struct.
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {ParsedEnumMember|undefined} id  The id of the enum.
 * @param  {ParserLocation} loc The location of the enum.
 * @param  {(string | PreprocessorStatement)[] | undefined} docstring The doc comment above the enum.
 * @param  {ParsedEnumMember[]} body  The members of the enum.
 * @returns void
 */
export function readEnumStruct(
  parserArgs: spParserArgs,
  id: ParsedEnumMember | undefined,
  loc: ParserLocation,
  docstring: (string | PreprocessorStatement)[] | undefined,
  body: ParsedEnumStructMember[]
): void {
  const { doc, dep } = processDocStringComment(docstring);
  const enumStructItem = new EnumStructItem(
    id.id,
    parserArgs.filePath,
    doc,
    parsedLocToRange(id.loc),
    parsedLocToRange(loc)
  );
  // TODO: Define separated enum members in the parser.
  parserArgs.fileItems.set(id.id, enumStructItem);
  body["body"].forEach((e: ParsedEnumStructMember) => {
    if (e["type"] === "FunctionDeclaration") {
      readFunctionAndMethod(
        parserArgs,
        e.accessModifier,
        e.returnType,
        e.id,
        e.loc,
        [""],
        e.params,
        e.body
      );
    } else if (e["type"] === "VariableDeclaration") {
      readVariable(parserArgs, e, enumStructItem);
    }
  });
}
