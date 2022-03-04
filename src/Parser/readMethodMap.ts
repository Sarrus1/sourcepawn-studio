import { Parser } from "./spParser";
import { State } from "./stateEnum";
import { parseDocComment } from "./parseDocComment";
import { MethodMapItem } from "../Backend/Items/spMethodmapItem";

export function readMethodMap(
  parser: Parser,
  match: RegExpMatchArray,
  line: string
): void {
  parser.state.push(State.Methodmap);
  parser.state_data = {
    name: match[1],
  };
  let { description, params } = parseDocComment(parser);
  let range = parser.makeDefinitionRange(match[1], line);
  var methodMapCompletion = new MethodMapItem(
    match[1],
    match[2],
    line.trim(),
    description,
    parser.filePath,
    range,
    parser.IsBuiltIn
  );
  parser.fileItems.set(match[1], methodMapCompletion);
}
