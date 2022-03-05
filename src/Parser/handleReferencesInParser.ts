import { URI } from "vscode-uri";
import { CompletionItemKind, Location, Position } from "vscode";

import { Parser } from "./spParser";
import { positiveRange } from "./utils";
import { SPItem } from "../Backend/Items/spItems";
import { globalIdentifier } from "../Misc/spConstants";
import { MethodItem } from "../Backend/Items/spMethodItem";
import { PropertyItem } from "../Backend/Items/spPropertyItem";

const globalScope = `-${globalIdentifier}-${globalIdentifier}`;

export function handleReferenceInParser(
  this: {
    parser: Parser;
    offset: number;
    previousItems: SPItem[];
    line: string;
    scope: string;
  },
  match: RegExpExecArray
) {
  const matchPosition = new Position(this.parser.lineNb, match.index + 1);

  if (match[0] === "this") {
    let item = this.parser.items.find(
      (e) =>
        [CompletionItemKind.Struct, CompletionItemKind.Class].includes(
          e.kind
        ) &&
        this.parser.filePath == e.filePath &&
        e.fullRange.contains(matchPosition)
    );
    if (item !== undefined) {
      this.previousItems.push(item);
    }
    return;
  }

  const item =
    this.parser.referencesMap.get(match[0] + this.scope) ||
    this.parser.referencesMap.get(match[0] + globalScope) ||
    this.parser.referencesMap.get(match[0]);

  if (item !== undefined) {
    const range = positiveRange(
      this.parser.lineNb,
      match.index + this.offset,
      match.index + match[0].length + this.offset
    );

    // Prevent double references.
    if (item.range.isEqual(range)) {
      return;
    }
    const location = new Location(URI.file(this.parser.filePath), range);
    item.references.push(location);
    this.previousItems.push(item);
  } else if (
    match.index > 0 &&
    this.previousItems.length > 0 &&
    [".", ":"].includes(this.line[match.index - 1])
  ) {
    let offset = 1;
    let item: MethodItem | PropertyItem;
    while (item === undefined && this.previousItems.length >= offset) {
      let parent = this.previousItems[this.previousItems.length - offset];
      item = this.parser.methodsAndProperties.find(
        (e) =>
          e.name === match[0] &&
          (e.parent.name === parent.type || e.parent === parent)
      );
      offset++;
    }

    if (item !== undefined) {
      const range = positiveRange(
        this.parser.lineNb,
        match.index + this.offset,
        match.index + match[0].length + this.offset
      );
      if (item.range.isEqual(range)) {
        return;
      }
      const location = new Location(URI.file(this.parser.filePath), range);
      item.references.push(location);
      this.previousItems.push(item);
    }
  }
}
