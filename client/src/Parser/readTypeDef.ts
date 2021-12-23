import { Parser } from "./spParser";
import { TypeDefItem } from "../Backend/spItems";
import { Range } from "vscode";
import { parseDocComment } from "./parseDocComment";

export function readTypeDef(
  parser: Parser,
  match: RegExpMatchArray,
  line: string
): void {
  let name = match[1];
  let type = match[2];
  let range = parser.makeDefinitionRange(name, line);
  let { description, params } = parseDocComment(parser);
  let fullRange = new Range(parser.lineNb, 0, parser.lineNb, line.length);
  parser.completions.set(
    name,
    new TypeDefItem(
      name,
      match[0],
      parser.file,
      description,
      type,
      range,
      fullRange
    )
  );
}
