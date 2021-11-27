import { Parser } from "./spParser";
import { PropertyItem } from "../Providers/spItems";
import { parseDocComment } from "./parseDocComment";

export function readProperty(
  parser: Parser,
  match: RegExpMatchArray,
  line: string
) {
  let { description, params } = parseDocComment(parser);
  let name_match: string = match[2];
  parser.lastFuncName = name_match;
  let range = parser.makeDefinitionRange(name_match, line);
  let NewPropertyCompletion = new PropertyItem(
    parser.state_data.name,
    name_match,
    parser.file,
    match[0],
    description,
    range,
    match[1]
  );
  parser.completions.add(
    name_match + parser.state_data.name,
    NewPropertyCompletion
  );
}
