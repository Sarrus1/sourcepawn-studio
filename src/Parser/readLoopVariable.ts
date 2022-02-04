import { Parser } from "./spParser";
import { State } from "./stateEnum";
import { addVariableItem } from "./addVariableItem";

export function readLoopVariable(
  parser: Parser,
  match: RegExpMatchArray,
  line: string
) {
  if (/\{\s*$/.test(line)) {
    parser.state.push(State.Loop);
  }
  // Test the next line if we didn't match
  else if (/^\s*\{/.test(parser.lines[0])) {
    parser.state.push(State.Loop);
  }
  if (parser.IsBuiltIn) {
    return;
  }
  addVariableItem(parser, match[1], line, "int");
  return;
}
