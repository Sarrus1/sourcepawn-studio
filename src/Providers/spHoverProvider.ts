import { TextDocument, Position, CancellationToken, Hover } from "vscode";
import { ItemsRepository } from "../Backend/spItemsRepository";
import { orderItems } from "./Hover/spHoverFilter";

export function hoverProvider(
  itemsRepo: ItemsRepository,
  document: TextDocument,
  position: Position,
  token: CancellationToken
): Hover | undefined {
  const items = itemsRepo.getItemFromPosition(document, position);
  orderItems(items);
  if (items.length > 0 && items[0].toHover()) {
    return items[0].toHover() as Hover;
  }
  return undefined;
}
