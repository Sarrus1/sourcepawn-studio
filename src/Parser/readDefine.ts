import { spParserArgs } from "./interfaces";
import { DefineItem } from "../Backend/Items/spDefineItem";
import { ParsedComment, ParsedID, ParserLocation } from "./interfaces";
import { parsedLocToRange } from "./utils";
import { processDocStringComment } from "./processComment";

/**
 * Callback for a parsed define.
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {ParsedID} id  The id of the define.
 * @param  {ParserLocation} loc  The location of the define.
 * @param  {string|null} value  The value of the define, if it exists.
 * @param  {string} docstring  The trailing documentation of the define.
 * @returns void
 */
export function readDefine(
  parserArgs: spParserArgs,
  id: ParsedID,
  loc: ParserLocation,
  value: string | null,
  docstring: ParsedComment
): void {
  const range = parsedLocToRange(id.loc, parserArgs);
  const fullRange = parsedLocToRange(loc, parserArgs);
  const { doc, dep } = processDocStringComment(docstring);
  const defineItem = new DefineItem(
    id.id,
    value ? value : "",
    doc,
    parserArgs.filePath,
    range,
    parserArgs.IsBuiltIn,
    fullRange
  );
  parserArgs.fileItems.items.push(defineItem);
}
