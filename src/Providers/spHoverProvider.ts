import { TextDocument, Position, CancellationToken, Hover } from "vscode";
import { ItemsRepository } from "../Backend/spItemsRepository";
import { orderItems } from "./Hover/spHoverFilter";

export function hoverProvider(
  itemsRepo: ItemsRepository,
  document: TextDocument,
  position: Position,
  token: CancellationToken
): Hover {
  let items = itemsRepo.getItemFromPosition(document, position);
  orderItems(items);
  if (items !== undefined && items.length > 0) {
    return items[0].toHover();
  }
  return undefined;
}
