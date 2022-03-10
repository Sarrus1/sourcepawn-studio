import { isIncludeSelfFile } from "./utils";
import { spParserArgs } from "./spParser";

export function readInclude(parserArgs: spParserArgs, txt: string) {
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
