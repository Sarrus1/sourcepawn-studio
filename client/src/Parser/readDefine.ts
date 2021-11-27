import { positiveRange } from "./utils";
import { Parser } from "./spParser";
import { DefineItem } from "../Providers/spItems";

export function readDefine(
  parser: Parser,
  match: RegExpMatchArray,
  line: string
): void {
  parser.definesMap.set(match[1], parser.file);
  let range = parser.makeDefinitionRange(match[1], line);
  let fullRange = positiveRange(parser.lineNb, 0, line.length);
  parser.completions.add(
    match[1],
    new DefineItem(
      match[1],
      match[2],
      parser.file,
      range,
      parser.IsBuiltIn,
      fullRange
    )
  );
  return;
}
