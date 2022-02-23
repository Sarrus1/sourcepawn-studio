import {
  TextDocument,
  CancellationToken,
  Location,
  ReferenceContext,
  Position,
  CompletionItemKind,
} from "vscode";
import { FunctionItem } from "../Backend/Items/spFunctionItem";
import { ItemsRepository } from "../Backend/spItemsRepository";
import { globalIdentifier } from "../Misc/spConstants";
import { positiveRange } from "../Parser/utils";
import { searchForReferencesInString } from "../Parser/searchForReferencesInString";
import { URI } from "vscode-uri";

export function referencesProvider(
  itemsRepo: ItemsRepository,
  position: Position,
  document: TextDocument,
  context: ReferenceContext,
  token: CancellationToken
): Location[] {
  const items = itemsRepo.getItemFromPosition(document, position);
  if (items.length === 0) {
    return [];
  }

  // Deal with local variables
  if (
    items.length === 1 &&
    items[0].kind === CompletionItemKind.Variable &&
    items[0].parent !== globalIdentifier
  ) {
    let references: Location[] = [];
    const allItems = itemsRepo.getAllItems(document.uri);
    const func = allItems.find(
      (e) =>
        e.kind === CompletionItemKind.Function && e.name === items[0].parent
    ) as FunctionItem;
    const text = document.getText(func.fullRange).split("\n");
    let lineNb = func.fullRange.start.line;
    for (let line of text) {
      searchForReferencesInString.call(
        {
          references: references,
          name: items[0].name,
          lineNb: lineNb,
          uri: document.uri,
        },
        line,
        handleReferencesInProvider
      );
      lineNb++;
    }
    return references;
  }

  return items
    .filter((e) => e.references !== undefined)
    .map((e) => e.references)
    .filter((e) => e !== undefined)
    .flat();
}
/**
 * Callback function for the `searchForReferencesInString` function when searching for local variable references.
 * @param  {{references:Location[];name:string;lineNb:number;uri:URI}} this
 * @param  {RegExpExecArray} match
 * @returns void
 */
function handleReferencesInProvider(
  this: { references: Location[]; name: string; lineNb: number; uri: URI },
  match: RegExpExecArray
): void {
  if (match[0] == this.name) {
    const range = positiveRange(
      this.lineNb,
      match.index,
      match.index + match[0].length
    );
    const location = new Location(this.uri, range);
    this.references.push(location);
  }
}
