import {
  TextDocument,
  CancellationToken,
  Location,
  ReferenceContext,
  Position,
} from "vscode";
import { ItemsRepository } from "../Backend/spItemsRepository";

export function referencesProvider(
  itemsRepo: ItemsRepository,
  position: Position,
  document: TextDocument,
  context: ReferenceContext,
  token: CancellationToken
): Location[] {
  const items = itemsRepo.getItemFromPosition(document, position);
  return items
    .filter((e) => e.references !== undefined)
    .map((e) => e.references)
    .filter((e) => e !== undefined)
    .flat();
}
