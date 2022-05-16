import { isIncludeSelfFile, parsedLocToRange } from "./utils";
import { spParserArgs, IncludeStatement } from "./interfaces";

/**
 * Process an include statement.
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {IncludeStatement} res  The parsed include statement.
 * @returns void
 */
export function readInclude(
  parserArgs: spParserArgs,
  res: IncludeStatement
): void {
  // Include guard to avoid extension crashs.
  if (isIncludeSelfFile(parserArgs.filePath, res.path)) {
    return;
  }
  parserArgs.fileItems.resolveImport(
    res.path,
    parserArgs.documents,
    parserArgs.filePath,
    parsedLocToRange(res.loc, parserArgs),
    parserArgs.IsBuiltIn
  );
}
