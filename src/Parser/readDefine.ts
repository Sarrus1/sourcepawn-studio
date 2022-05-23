import { DefineStatement, spParserArgs } from "./interfaces";
import { DefineItem } from "../Backend/Items/spDefineItem";
import { parsedLocToRange } from "./utils";
import { processDocStringComment } from "./processComment";

/**
 * Process a define statement.
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {DefineStatement} res  The object containing the define statement details.
 * @returns void
 */
export function readDefine(
  parserArgs: spParserArgs,
  res: DefineStatement
): void {
  const range = parsedLocToRange(res.id.loc, parserArgs);
  const fullRange = parsedLocToRange(res.loc, parserArgs);
  // FIXME: Define dep is always null because they are parsed as comments.
  const { doc, dep } = processDocStringComment(res.doc);
  const defineItem = new DefineItem(
    res.id.id,
    res.value !== null ? res.value.trim() : "",
    doc,
    parserArgs.filePath,
    range,
    parserArgs.IsBuiltIn,
    fullRange
  );
  parserArgs.fileItems.items.push(defineItem);
}
