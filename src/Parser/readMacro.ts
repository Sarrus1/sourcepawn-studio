import { Parser } from "./spParser";
import { MacroItem } from "../Backend/Items/spMacroItem";
import { parseDocComment } from "./parseDocComment";

export function readMacro(
  parser: Parser,
  match: RegExpMatchArray,
  line: string
): void {
  let { description, params } = parseDocComment(parser);
  let nameMatch = match[1];
  let details = `${nameMatch}(${match[2]})`;
  let range = parser.makeDefinitionRange(nameMatch, line);
  // Add the macro to the array of known macros
  parser.macroArr.push(nameMatch);
  parser.fileItems.set(
    nameMatch,
    new MacroItem(
      nameMatch,
      details,
      description,
      params,
      parser.filePath,
      parser.IsBuiltIn,
      range,
      "",
      undefined,
      undefined
    )
  );
}
