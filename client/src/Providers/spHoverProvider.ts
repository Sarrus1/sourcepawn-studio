import { TextDocument, Position, CancellationToken, Hover } from "vscode";
import { ItemsRepository } from "../Backend/spItemsRepository";

export function hoverProvider(
  itemsRepo: ItemsRepository,
  document: TextDocument,
  position: Position,
  token: CancellationToken
): Hover {
  let items = itemsRepo.getItemFromPosition(document, position);
  if (items.length > 0) {
    return items[0].toHover();
  }
  return undefined;
}
