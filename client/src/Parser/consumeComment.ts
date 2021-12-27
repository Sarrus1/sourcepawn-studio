import { Parser } from "./spParser";
import { CommentItem } from "../Backend/spItems";
import { Range, Position } from "vscode";
import { searchForDefinesInString } from "./searchForDefinesInString";

export function consumeComment(
  parser: Parser,
  current_line: string,
  use_line_comment: boolean = false
): void {
  parser.scratch = [];
  let startPos = new Position(parser.lineNb < 1 ? 0 : parser.lineNb, 0);
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
  let endPos = new Position(
    parser.lineNb < 2
      ? 0
      : use_line_comment
      ? parser.lineNb - 1
      : parser.lineNb - 2,
    current_line.length
  );
  let range = new Range(startPos, endPos);
  parser.completions.set(
    `comment${parser.lineNb}--${Math.random()}`,
    new CommentItem(parser.file, range)
  );
  searchForDefinesInString(parser, current_line);
  parser.interpLine(current_line);
  return;
}
