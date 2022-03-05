import {
  TextDocument,
  CancellationToken,
  Location,
  ReferenceContext,
  Position,
} from "vscode";
import { ItemsRepository } from "../Backend/spItemsRepository";
import { positiveRange } from "../Parser/utils";
import { URI } from "vscode-uri";
import { locationFromRange } from "../spUtils";

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

  const references = items
    .filter((e) => e.references !== undefined)
    .map((e) => e.references)
    .filter((e) => e !== undefined)
    .flat();

  if (context.includeDeclaration) {
    references.push(
      ...items.map((e) => locationFromRange(e.filePath, e.range))
    );
  }

  return references;
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
