import * as TreeSitter from "web-tree-sitter";

import { globalItem } from "../Misc/spConstants";
import { VariableItem } from "../Backend/Items/spVariableItem";
import { pointsToRange } from "./utils";
import { TreeWalker } from "./spParser";

/**
 * Process a global variable declaration.
 */
export function readVariable(
  walker: TreeWalker,
  node: TreeSitter.SyntaxNode
): void {
  const variableType = node.childForFieldName("type").text;
  let storageClass = [];
  for (let child of node.children) {
    // FIXME: More efficient way to do this ?
    // FIXME: Old declarations are broken with tree-sitter-sourcepawn.
    if (child.type === "variable_storage_class") {
      // FIXME: Only works for 0 or 1 storage class.
      // This has to be fixed in tree-sitter-sourcepawn.
      storageClass.push(child.text);
      continue;
    }
    if (child.type !== "variable_declaration") {
      continue;
    }
    const declaration = child.childForFieldName("name");
    const variableItem = new VariableItem(
      declaration.text,
      walker.filePath,
      globalItem,
      pointsToRange(declaration.startPosition, declaration.endPosition),
      variableType,
      // TODO: Handle comments.
      "detail",
      "doc",
      storageClass
    );
    walker.fileItem.items.push(variableItem);
  }
}
