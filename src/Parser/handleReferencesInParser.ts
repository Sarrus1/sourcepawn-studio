import { URI } from "vscode-uri";
import { CompletionItemKind, Location, Position } from "vscode";

import { Parser } from "./spParser";
import { positiveRange } from "./utils";
import { SPItem } from "../Backend/Items/spItems";
import { globalIdentifier } from "../Misc/spConstants";

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
  const globalScope = `-${globalIdentifier}-${globalIdentifier}`;

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
    let parent = this.previousItems[this.previousItems.length - 1];

    let item = this.parser.methodsAndProperties.find(
      (e) =>
        [CompletionItemKind.Property, CompletionItemKind.Method].includes(
          e.kind
        ) &&
        e.name === match[0] &&
        (e.parent === parent.type || e.parent === parent.name)
    );

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
