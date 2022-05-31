import { FunctionParam, MacroStatement, spParserArgs } from "./interfaces";

/**
 * Process a macro statement.
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {MacroStatement} res  Object containing the macro statement details.
 * @returns void
 */
export function readMacro(parserArgs: spParserArgs, res: MacroStatement): void {
  return;
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
