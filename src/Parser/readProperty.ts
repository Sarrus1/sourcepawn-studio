import * as TreeSitter from "web-tree-sitter";

import { PropertyItem } from "../Backend/Items/spPropertyItem";
import { MethodMapItem } from "../Backend/Items/spMethodmapItem";
import { TreeWalker } from "./spParser";
import { findDoc } from "./readDocumentation";
import { pointsToRange } from "./utils";
import {
  readBodyVariables,
  readFunctionAndMethod,
} from "./readFunctionAndMethod";
import { MethodItem } from "../Backend/Items/spMethodItem";

/**
 * Process a methodmap's property.
 * @param  {TreeWalker} walker            TreeWalker object.
 * @param  {TreeSitter.SyntaxNode} node   Node to process.
 * @param  {MethodMapItem} parent         Parent item of the property.
 * @returns void
 */
export function readProperty(
  walker: TreeWalker,
  node: TreeSitter.SyntaxNode,
  parent: MethodMapItem
): void {
  const nameNode = node.childForFieldName("name");
  const typeNode = node.childForFieldName("type");
  const { doc, dep } = findDoc(walker, node);
  const propertyItem = new PropertyItem(
    parent,
    nameNode.text,
    walker.filePath,
    `property ${typeNode.text} ${nameNode.text}`,
    doc,
    pointsToRange(nameNode.startPosition, nameNode.endPosition),
    pointsToRange(node.startPosition, node.endPosition),
    typeNode.text
  );
  walker.fileItem.items.push(propertyItem);
  node.children.forEach((e1) => {
    if (!e1.type.startsWith("methodmap_property_")) {
      return;
    }
    e1.children.forEach((e2) => {
      switch (e2.type) {
        case "methodmap_property_getter":
          readFunctionAndMethod(walker, e2, propertyItem, "get");
          break;
        case "methodmap_property_setter":
          readFunctionAndMethod(walker, e2, propertyItem, "set");
          break;
        case "block":
          // Properties's methods bodies are not children of the methods in the tree-sitter AST.
          // This addresses this design choice, however it should be adressed.
          const lastIdx = walker.fileItem.items.length - 1;
          const lastItem = walker.fileItem.items[lastIdx] as MethodItem;
          readBodyVariables(walker, e2, lastItem);
          const bodyRange = pointsToRange(e2.startPosition, e2.endPosition);
          lastItem.fullRange = lastItem.fullRange.union(bodyRange);
          break;
        default:
          break;
      }
    });
  });
}
