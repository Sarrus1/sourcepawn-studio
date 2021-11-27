import { Parser } from "./spParser";
import { State } from "./stateEnum";
import { parseDocComment } from "./parseDocComment";
import { MethodMapItem } from "../Providers/spItems";

export function readMethodMap(
  parser: Parser,
  match: RegExpMatchArray,
  line: string
): void {
  parser.state.push(State.Methodmap);
  parser.state_data = {
    name: match[1],
  };
  let { description, params } = parseDocComment(this);
  let range = parser.makeDefinitionRange(match[1], line);
  var methodMapCompletion = new MethodMapItem(
    match[1],
    match[2],
    line.trim(),
    description,
    parser.file,
    range,
    parser.IsBuiltIn
  );
  parser.completions.add(match[1], methodMapCompletion);
}
