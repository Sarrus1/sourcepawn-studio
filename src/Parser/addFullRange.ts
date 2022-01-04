import { Parser } from "./spParser";
import { Range } from "vscode";

export function addFullRange(parser: Parser, key: string) {
  let completion = parser.completions.get(key);
  if (completion && completion.fullRange === undefined) {
    let range = completion.range;
    let fullRange = new Range(
      range.start.line,
      range.start.character,
      parser.lineNb,
      1
    );
    completion.fullRange = fullRange;
    parser.completions.set(key, completion);
  }
}
