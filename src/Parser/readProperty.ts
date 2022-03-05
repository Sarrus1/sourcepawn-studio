import { Parser } from "./spParser";
import { PropertyItem } from "../Backend/Items/spPropertyItem";
import { parseDocComment } from "./parseDocComment";
import { MethodMapItem } from "../Backend/Items/spMethodmapItem";
import { EnumStructItem } from "../Backend/Items/spEnumStructItem";

export function readProperty(
  parser: Parser,
  match: RegExpMatchArray,
  line: string
) {
  let { description, params } = parseDocComment(parser);
  let name_match: string = match[2];
  let range = parser.makeDefinitionRange(name_match, line);
  let propertyItem = new PropertyItem(
    parser.fileItems.get(parser.state_data.name) as
      | MethodMapItem
      | EnumStructItem,
    name_match,
    parser.filePath,
    match[0],
    description,
    range,
    match[1]
  );
  parser.lastFunc = propertyItem;
  parser.fileItems.set(name_match + parser.state_data.name, propertyItem);
}
