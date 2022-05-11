import { isIncludeSelfFile, parsedLocToRange } from "./utils";
import { spParserArgs, IncludeStatement } from "./interfaces";

/**
 * Callback for a parsed include.
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {IncludeStatement} include  The parsed include statement.
 * @returns void
 */
export function readInclude(
  parserArgs: spParserArgs,
  include: IncludeStatement
): void {
  // Include guard to avoid extension crashs.
  if (isIncludeSelfFile(parserArgs.filePath, include.path)) {
    return;
  }
  parserArgs.fileItems.resolveImport(
    include.path,
    parserArgs.documents,
    parserArgs.filePath,
    parsedLocToRange(include.loc, parserArgs),
    parserArgs.IsBuiltIn
  );
  return;
}
