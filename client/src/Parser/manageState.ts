import { Parser } from "./spParser";
import { State } from "./stateEnum";

export function manageState(parser: Parser, line: string): void {
  if (/^\s*\}\s*\belse\b\s*\{/.test(line)) {
    return;
  }
  let state = parser.state[parser.state.length - 1];
  if (state === State.None) {
  } else if (state === State.Function && parser.state_data !== undefined) {
    // We are in a method
    parser.lastFuncLine = 0;
    parser.addFullRange(parser.lastFuncName + parser.state_data.name);
  } else if (state === State.Methodmap && parser.state_data !== undefined) {
    // We are in a methodmap
    parser.addFullRange(parser.state_data.name);
    parser.state_data = undefined;
  } else if (state === State.EnumStruct && parser.state_data !== undefined) {
    // We are in an enum struct
    parser.addFullRange(parser.state_data.name);
    parser.state_data = undefined;
  } else if (state === State.Property && parser.state_data !== undefined) {
    // We are in a property
    parser.addFullRange(parser.lastFuncName + parser.state_data.name);
  } else if (
    ![
      State.Methodmap,
      State.EnumStruct,
      State.Property,
      State.Loop,
      State.Macro,
    ].includes(state)
  ) {
    // We are in a regular function
    parser.addFullRange(parser.lastFuncName);
  }
  parser.state.pop();
}
