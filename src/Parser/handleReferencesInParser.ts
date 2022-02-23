import { Parser } from "./spParser";
import { positiveRange } from "./utils";
import { URI } from "vscode-uri";
import { Location } from "vscode";

export function handleReferenceInParser(
  this: { parser: Parser; offset: number },
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
  }
}
