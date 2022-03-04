import { isIncludeSelfFile } from "./utils";
import { Parser } from "./spParser";

export function readInclude(parser: Parser, match: RegExpMatchArray) {
  // Include guard to avoid extension crashs.
  if (isIncludeSelfFile(parser.filePath, match[1])) return;
  parser.fileItems.resolveImport(
    match[1],
    parser.documents,
    parser.filePath,
    parser.IsBuiltIn
  );
  return;
}
