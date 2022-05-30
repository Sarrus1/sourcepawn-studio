import * as TreeSitter from "web-tree-sitter";

import { globalItem } from "../Misc/spConstants";
import { VariableItem } from "../Backend/Items/spVariableItem";
import { pointsToRange } from "./utils";
import { TreeWalker } from "./spParser";
import { ConstantItem } from "../Backend/Items/spConstantItem";
import { EnumStructItem } from "../Backend/Items/spEnumStructItem";
import { FunctionItem } from "../Backend/Items/spFunctionItem";
import { MethodItem } from "../Backend/Items/spMethodItem";

export type VariableParent =
  | ConstantItem
  | EnumStructItem
  | FunctionItem
  | MethodItem;

/**
 * Process a variable declaration.
 * @param  {TreeWalker} walker            TreeWalker used to find the documentation.
 * @param  {TreeSitter.SyntaxNode} node   Node to process.
 * @param  {VariableParent} parent        Parent of the variable. Defaults to globalItem.
 * @returns void
 */
export function readVariable(
  walker: TreeWalker,
  node: TreeSitter.SyntaxNode,
  parent: VariableParent = globalItem
): void {
  const variableType = node.childForFieldName("type").text;
  let storageClass: string[] = [];
  for (let child of node.children) {
    // FIXME: More efficient way to do this ?
    // FIXME: Old declarations are broken with tree-sitter-sourcepawn. Needs to be investigated.
    // FIXME: `public` does not seem to be allowed as a storage class.
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
      "",
      storageClass
    );
    walker.fileItem.items.push(variableItem);
  }
}
