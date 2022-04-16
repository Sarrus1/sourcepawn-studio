import { MacroItem } from "../Backend/Items/spMacroItem";
import { spParserArgs } from "./spParser";
import { ParsedComment, ParsedID, ParserLocation } from "./interfaces";
import { parsedLocToRange } from "./utils";
import { processDocStringComment } from "./processComment";

/**
 * Callback for a parsed macro.
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {ParsedID} id  The id of the macro.
 * @param  {ParserLocation} loc  The location of the macro.
 * @param  {string|null} value  The value of the macro, if it exists.
 * @param  {ParsedComment} docstring  The documentation of the macro.
 * @returns void
 */
export function readMacro(
  parserArgs: spParserArgs,
  id: ParsedID,
  loc: ParserLocation,
  value: string | null,
  docstring: ParsedComment
): void {
  const range = parsedLocToRange(id.loc, parserArgs);
  const fullRange = parsedLocToRange(loc, parserArgs);
  const { doc, dep } = processDocStringComment(docstring);
  const macroItem = new MacroItem(
    id.id,
    value,
    doc,
    [],
    parserArgs.filePath,
    parserArgs.IsBuiltIn,
    range,
    undefined,
    fullRange,
    dep,
    undefined
  );
  parserArgs.fileItems.set(id.id, macroItem);
  return;
}
