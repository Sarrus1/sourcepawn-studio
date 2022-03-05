import { Parser } from "./spParser";
import { State } from "./stateEnum";
import { parseDocComment } from "./parseDocComment";
import { MethodMapItem } from "../Backend/Items/spMethodmapItem";
import { CompletionItemKind } from "vscode";
import { globalItem } from "../Misc/spConstants";
import { ConstantItem } from "../Backend/Items/spConstantItem";

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
  let parent =
    parser.items.find(
      (e) => e.kind === CompletionItemKind.Class && e.name === match[2]
    ) ||
    parser.fileItems.get(match[2]) ||
    globalItem;
  var methodMapCompletion = new MethodMapItem(
    match[1],
    parent as MethodMapItem | ConstantItem,
    line.trim(),
    description,
    parser.filePath,
    range,
    parser.IsBuiltIn
  );
  parser.fileItems.set(match[1], methodMapCompletion);
}
