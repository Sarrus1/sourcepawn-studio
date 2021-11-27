import { Parser } from "./spParser";
import { TypeSetItem } from "../Providers/spItems";
import { Range, Position } from "vscode";

export function readTypeSet(
  parser: Parser,
  match: RegExpMatchArray,
  line: string
): void {
  let startPosition = new Position(parser.lineNb, 0);
  let name = match[1];
  let range = parser.makeDefinitionRange(name, line);
  let { description, params } = parser.parse_doc_comment();
  let iter = 0;
  while (!/^\s*}/.test(line)) {
    if (iter == 200) {
      return;
    }
    line = parser.lines.shift();
    parser.lineNb++;
    parser.searchForDefinesInString(line);
    iter++;
  }
  let endMatch = line.match(/^\s*}/);
  let fullRange = new Range(
    startPosition,
    new Position(parser.lineNb, endMatch[0].length)
  );
  parser.completions.add(
    name,
    new TypeSetItem(name, match[0], parser.file, description, range, fullRange)
  );
}
