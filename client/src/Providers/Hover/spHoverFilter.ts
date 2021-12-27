import { SPItem } from "../../Backend/Items/spItems";

/**
 * Sort an array of items based on the length of their description.
 * @param  {SPItem[]} items
 * @returns void
 */
export function orderItems(items: SPItem[]): void {
  if (items === undefined || items.length < 2) {
    return;
  }
  items.sort((a, b) => {
    if (a.description === undefined) {
      return 1;
    }
    if (b.description === undefined) {
      return -1;
    }
    return b.description.length - a.description.length;
  });
}
