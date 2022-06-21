import {
  TextDocument,
  CancellationToken,
  Location,
  ReferenceContext,
  Position,
} from "vscode";
import { ItemsRepository } from "../Backend/spItemsRepository";
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
