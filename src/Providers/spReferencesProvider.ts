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

export function referencesProvider(
  itemsRepo: ItemsRepository,
  position: Position,
  document: TextDocument,
  context: ReferenceContext,
  token: CancellationToken
): Location[] {
  const items = itemsRepo.getItemFromPosition(document, position);
  if (items.length > 0) {
    return items
      .filter((e) => e.kind === CompletionItemKind.Function)
      .map((e: FunctionItem) => e.toReferenceItem())
      .filter((e) => e !== undefined)
      .flat() as Location[];
  }
  return [];
}
