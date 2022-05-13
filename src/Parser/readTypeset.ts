import { spParserArgs } from "./interfaces";
import { TypesetItem } from "../Backend/Items/spTypesetItem";
import { ParsedComment, ParsedID, ParserLocation } from "./interfaces";
import { parsedLocToRange } from "./utils";
import { processDocStringComment } from "./processComment";

/**
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {ParsedID} id  The id of the Typeset.
 * @param  {ParserLocation} loc  The location of the Typeset.
 * @param  {ParsedComment} docstring  The documentation of the Typeset.
 * @returns void
 */
export function readTypeset(
  parserArgs: spParserArgs,
  id: ParsedID,
  loc: ParserLocation,
  docstring: ParsedComment
): void {
  const range = parsedLocToRange(id.loc, parserArgs);
  const fullRange = parsedLocToRange(loc, parserArgs);
  const { doc, dep } = processDocStringComment(docstring);
  const typeDefItem = new TypesetItem(
    id.id,
    `typeset ${id.id}`,
    parserArgs.filePath,
    doc,
    range,
    fullRange
  );
  parserArgs.fileItems.items.push(typeDefItem);
  return;
}
