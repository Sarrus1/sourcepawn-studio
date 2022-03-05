import {
  TextDocument,
  CancellationToken,
  Position,
  WorkspaceEdit,
} from "vscode";
import { URI } from "vscode-uri";

import { ItemsRepository } from "../Backend/spItemsRepository";

export function renameProvider(
  itemsRepo: ItemsRepository,
  position: Position,
  document: TextDocument,
  newText: string,
  token: CancellationToken
): WorkspaceEdit | undefined {
  const items = itemsRepo.getItemFromPosition(document, position);
  if (items.length === 0) {
    return undefined;
  }

  const edit = new WorkspaceEdit();
  items.forEach((e1) => {
    if (e1.references !== undefined) {
      e1.references.forEach((e2) => {
        edit.replace(e2.uri, e2.range, newText);
      });
    }
    edit.replace(URI.file(e1.filePath), e1.range, newText);
  });
  return edit;
}
