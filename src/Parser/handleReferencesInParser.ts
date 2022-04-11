import { URI } from "vscode-uri";
import { CompletionItemKind, Location, Range } from "vscode";

import { Parser } from "./spParser";
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
    lineNb: number;
    scope: string;
    outsideScope: string;
  },
  name: string,
  range: Range
) {
  const start = range.start.character;

  if (name === "this") {
    let item = this.parser.items.find(
      (e) =>
        [CompletionItemKind.Struct, CompletionItemKind.Class].includes(
          e.kind
        ) &&
        this.parser.filePath == e.filePath &&
        e.fullRange.contains(range)
    );
    if (item !== undefined) {
      this.previousItems.push(item);
    }
    return;
  }

  const item =
    this.parser.referencesMap.get(name + this.scope) ||
    this.parser.referencesMap.get(name + this.outsideScope) ||
    this.parser.referencesMap.get(name + globalScope) ||
    this.parser.referencesMap.get(name);

  if (item !== undefined) {
    // Prevent double references.
    if (item.range.isEqual(range)) {
      return;
    }
    const location = new Location(URI.file(this.parser.filePath), range);
    item.references.push(location);
    this.previousItems.push(item);
  } else if (
    start > 0 &&
    this.previousItems.length > 0 &&
    [".", ":"].includes(this.line[start - 1])
  ) {
    let offset = 1;
    let item: MethodItem | PropertyItem;
    while (item === undefined && this.previousItems.length >= offset) {
      let parent = this.previousItems[this.previousItems.length - offset];
      item = this.parser.methodsAndProperties.find(
        (e) =>
          e.name === name &&
          (e.parent.name === parent.type || e.parent === parent)
      );
      offset++;
    }

    if (item !== undefined) {
      if (item.range.isEqual(range)) {
        return;
      }
      const location = new Location(URI.file(this.parser.filePath), range);
      item.references.push(location);
      this.previousItems.push(item);
    }
  }
}
