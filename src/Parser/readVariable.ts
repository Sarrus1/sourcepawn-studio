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
  node: TreeSitter.SyntaxNode,
  parent = globalItem
): void {
  const variableType = node.childForFieldName("type").text;
  let storageClass: string[] = [];
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
      parent,
      pointsToRange(declaration.startPosition, declaration.endPosition),
      variableType,
      // TODO: Handle doc comments.
      `${storageClass.join(" ")} ${variableType} ${declaration.text}`,
      "doc",
      storageClass
    );
    walker.fileItem.items.push(variableItem);
  }
}
