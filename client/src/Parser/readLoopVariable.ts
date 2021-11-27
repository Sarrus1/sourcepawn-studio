import { Parser } from "./spParser";
import { State } from "./stateEnum";

export function readLoopVariable(
  parser: Parser,
  match: RegExpMatchArray,
  line: string
) {
  parser.state.push(State.Loop);
  if (parser.IsBuiltIn) {
    return;
  }
  parser.AddVariableCompletion(match[1], line, "int");
  return;
}
