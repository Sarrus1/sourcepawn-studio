import { MacroItem } from "../Backend/Items/spMacroItem";
import { FunctionParam, MacroStatement, spParserArgs } from "./interfaces";
import { parsedLocToRange } from "./utils";
import { processDocStringComment } from "./processComment";

/**
 * Process a macro statement.
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {MacroStatement} res  Object containing the macro statement details.
 * @returns void
 */
export function readMacro(parserArgs: spParserArgs, res: MacroStatement): void {
  const range = parsedLocToRange(res.id.loc, parserArgs);
  const fullRange = parsedLocToRange(res.loc, parserArgs);
  // FIXME: Macro docs are always empty because they are parsed as comments.
  const { doc, dep } = processDocStringComment(res.doc);
  const processedParams = processMacroParams(res.value, doc);
  const macroItem = new MacroItem(
    res.id.id,
    res.value,
    doc,
    processedParams,
    parserArgs.filePath,
    parserArgs.IsBuiltIn,
    range,
    undefined,
    fullRange,
    dep,
    undefined,
    undefined
  );
  parserArgs.fileItems.items.push(macroItem);
}

/**
 * Link docstring to a macro's params.
 * @param  {string|null} value  The raw string of the macro's body.
 * @param  {string|undefined} doc  The documentation of the macro.
 * @returns FunctionParam
 */
function processMacroParams(
  value: string | null,
  doc: string | undefined
): FunctionParam[] {
  if (!value) {
    return [];
  }
  const match = value.match(/\%\d/g);
  if (!match || match.length === 0) {
    return [];
  }
  const params = match.slice(1);
  const processedParams = params.map((e) => {
    let documentation = "";
    if (doc) {
      const match = doc.match(new RegExp(`@param\\s+(?:\\b${e}\\b)([^\\@]+)`));
      if (match) {
        documentation = match[1].replace(/\*/gm, "").trim();
      }
    }
    return {
      label: e,
      documentation,
    } as FunctionParam;
  });
  return processedParams;
}
