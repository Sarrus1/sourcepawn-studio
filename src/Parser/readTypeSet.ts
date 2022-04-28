import { spParserArgs } from "./interfaces";
import { TypeSetItem } from "../Backend/Items/spTypesetItem";
import { ParsedComment, ParsedID, ParserLocation } from "./interfaces";
import { parsedLocToRange } from "./utils";
import { processDocStringComment } from "./processComment";

/**
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {ParsedID} id  The id of the TypeSet.
 * @param  {ParserLocation} loc  The location of the TypeSet.
 * @param  {ParsedComment} docstring  The documentation of the TypeSet.
 * @returns void
 */
export function readTypeSet(
  parserArgs: spParserArgs,
  id: ParsedID,
  loc: ParserLocation,
  docstring: ParsedComment
): void {
  const range = parsedLocToRange(id.loc, parserArgs);
  const fullRange = parsedLocToRange(loc, parserArgs);
  const { doc, dep } = processDocStringComment(docstring);
  const typeDefItem = new TypeSetItem(
    id.id,
    "",
    parserArgs.filePath,
    doc,
    range,
    fullRange
  );
  parserArgs.fileItems.items.push(typeDefItem);
  return;
}
