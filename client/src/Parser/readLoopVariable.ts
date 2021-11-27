import { Parser } from "./spParser";
import { State } from "./stateEnum";
import { addVariableItem } from "./addVariableItem";

export function readLoopVariable(
  parser: Parser,
  match: RegExpMatchArray,
  line: string
) {
  parser.state.push(State.Loop);
  if (parser.IsBuiltIn) {
    return;
  }
  addVariableItem(parser, match[1], line, "int");
  return;
}
