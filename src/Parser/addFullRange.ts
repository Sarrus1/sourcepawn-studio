import { Parser } from "./spParser";
import { Range } from "vscode";

export function addFullRange(parser: Parser, key: string) {
  let item = parser.fileItems.get(key);
  if (item !== undefined) {
    item.fullRange = new Range(
      item.fullRange === undefined
        ? item.range.start.line
        : item.fullRange.start.line,
      item.fullRange === undefined
        ? item.range.start.character
        : item.fullRange.start.character,
      parser.lineNb,
      1
    );
  }
}
