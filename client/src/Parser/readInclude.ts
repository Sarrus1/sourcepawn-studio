import { isIncludeSelfFile } from "./utils";
import { Parser } from "./spParser";

export function readInclude(parser: Parser, match: RegExpMatchArray) {
  // Include guard to avoid extension crashs.
  if (isIncludeSelfFile(parser.file, match[1])) return;
  parser.completions.resolveImport(
    match[1],
    parser.documents,
    parser.file,
    parser.IsBuiltIn
  );
  return;
}
