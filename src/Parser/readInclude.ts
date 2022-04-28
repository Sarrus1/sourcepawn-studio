import { isIncludeSelfFile } from "./utils";
import { spParserArgs } from "./interfaces";

/**
 * Callback for a parsed include.
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {string} txt  The parsed text of the include.
 * @returns void
 */
export function readInclude(parserArgs: spParserArgs, txt: string): void {
  // Include guard to avoid extension crashs.
  if (isIncludeSelfFile(parserArgs.filePath, txt)) {
    return;
  }
  parserArgs.fileItems.resolveImport(
    txt,
    parserArgs.documents,
    parserArgs.filePath,
    parserArgs.IsBuiltIn
  );
  return;
}
