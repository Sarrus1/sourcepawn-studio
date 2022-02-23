import { Parser } from "./spParser";
import { positiveRange } from "./utils";
import { URI } from "vscode-uri";
import { CompletionItemKind, Location } from "vscode";
import { SPItem } from "../Backend/Items/spItems";

export function handleReferenceInParser(
  this: {
    parser: Parser;
    offset: number;
    previousItems: SPItem[];
    line: string;
  },
  match: RegExpExecArray
) {
  let item = this.parser.referencesMap.get(match[0]);

  if (item !== undefined) {
    const range = positiveRange(
      this.parser.lineNb,
      match.index + this.offset,
      match.index + match[0].length + this.offset
    );
    const location = new Location(URI.file(this.parser.file), range);
    item.references.push(location);
    this.previousItems.push(item);
  } else {
    if (
      match.index > 0 &&
      this.previousItems.length > 0 &&
      [".", ":"].includes(this.line[match.index - 1])
    ) {
      let parent = this.previousItems[this.previousItems.length - 1];
      let item = this.parser.items.find(
        (e) =>
          [CompletionItemKind.Property, CompletionItemKind.Method].includes(
            e.kind
          ) &&
          e.name === match[0] &&
          e.parent === parent.type
      );

      if (item !== undefined) {
        const range = positiveRange(
          this.parser.lineNb,
          match.index + this.offset,
          match.index + match[0].length + this.offset
        );
        const location = new Location(URI.file(this.parser.file), range);
        item.references.push(location);
        this.previousItems.push(item);
      }
    }
  }
}
