import { Parser } from "./spParser";
import { VariableItem } from "../Backend/Items/spVariableItem";
import { PropertyItem } from "../Backend/Items/spPropertyItem";
import { State } from "./stateEnum";
import { globalIdentifier, globalItem } from "../Misc/spConstants";
import { MethodMapItem } from "../Backend/Items/spMethodmapItem";
import { EnumStructItem } from "../Backend/Items/spEnumStructItem";

/**
 * Save a variable and generate the appropriate key for the Map it is stored in.
 * The key is a concatenation of the following variables:
 * - *varName*: The name of the variable
 * - *scope*: The scope of the variable (the last function's name or globalIdentifier)
 * - *enumStructName*: The name of the enum struct (empty if none)
 * - *lastFuncName*: The name of the last function (empty if none)
 * @param  {Parser} parser
 * @param  {string} name
 * @param  {string} line
 * @param  {string} type
 * @param  {string} funcName?
 * @param  {} isParamDef=false
 * @returns void
 */
export function addVariableItem(
  parser: Parser,
  name: string,
  line: string,
  type: string,
  isParamDef = false
): void {
  if (line === undefined) {
    return;
  }
  let range = parser.makeDefinitionRange(name, line);
  let scope = globalItem;
  let enumStructName: string;
  if (parser.state.includes(State.EnumStruct)) {
    enumStructName = parser.state_data.name;
  }
  if (parser.lastFuncLine !== -1) {
    scope = parser.lastFunc;
  }

  // Custom key name for the map so the definitions don't override each others
  let mapName = name + scope.name + enumStructName;
  if (
    (parser.state.includes(State.EnumStruct) ||
      parser.state.includes(State.Methodmap)) &&
    (parser.state.includes(State.Function) || isParamDef)
  ) {
    parser.fileItems.set(
      mapName + parser.lastFunc.name,
      new VariableItem(
        name,
        parser.filePath,
        parser.lastFunc,
        range,
        type,
        enumStructName
      )
    );
  } else if (parser.state.includes(State.EnumStruct)) {
    parser.fileItems.set(
      mapName,
      new PropertyItem(
        parser.fileItems.get(parser.state_data.name) as
          | MethodMapItem
          | EnumStructItem,
        name,
        parser.filePath,
        line,
        "",
        range,
        type
      )
    );
  } else {
    parser.fileItems.set(
      mapName,
      new VariableItem(
        name,
        parser.filePath,
        parser.lastFunc,
        range,
        type,
        globalIdentifier
      )
    );
  }
}
