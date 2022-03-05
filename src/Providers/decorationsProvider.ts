import { window, DecorationRenderOptions } from "vscode";

import { ItemsRepository } from "../Backend/spItemsRepository";

/**
 * Update the deprecated decorations of the currently active textEditor.
 * @param  {ItemsRepository} itemsRepo  The itemsRepository object constructed in the activation event.
 */
export async function updateDecorations(itemsRepo: ItemsRepository) {
  const editor = window.activeTextEditor;
  const allItems = itemsRepo.getAllItems(editor.document.uri);
  const options: DecorationRenderOptions = { textDecoration: "line-through" };

  const decorations = allItems
    .filter((e) => e.deprecated)
    .map((e1) =>
      e1.references
        .filter((e2) => e2.uri.fsPath === editor.document.uri.fsPath)
        .map((e3) => e3.range)
        .concat(e1.range)
    )
    .flat();

  editor.setDecorations(
    window.createTextEditorDecorationType(options),
    decorations
  );
}
