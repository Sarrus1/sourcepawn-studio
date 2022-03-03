import { Parser } from "./spParser";
import { Range, Position } from "vscode";

export function consumeComment(
  parser: Parser,
  current_line: string,
  use_line_comment: boolean = false
): void {
  parser.scratch = [];
  let iter = 0;
  while (
    current_line !== undefined &&
    iter < 100 &&
    ((/^\s*\/\//.test(current_line) && use_line_comment) ||
      (!/\*\//.test(current_line) && !use_line_comment))
  ) {
    iter++;
    parser.scratch.push(current_line.replace(/^\s*\/\//, "") + "\n");
    current_line = parser.lines.shift();

    parser.lineNb++;
  }
  // Removes the */ from the doc comment
  if (!use_line_comment) {
    current_line = parser.lines.shift();
    parser.lineNb++;
  }
  if (current_line === undefined) {
    return;
  }
  parser.interpLine(current_line);
  return;
}
