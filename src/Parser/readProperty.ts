import * as TreeSitter from "web-tree-sitter";

import { PropertyItem } from "../Backend/Items/spPropertyItem";
import { MethodMapItem } from "../Backend/Items/spMethodmapItem";
import { TreeWalker } from "./spParser";
import { findDoc } from "./readDocumentation";
import { pointsToRange } from "./utils";
import { readFunctionAndMethod } from "./readFunctionAndMethod";

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
    "",
    doc,
    pointsToRange(nameNode.startPosition, nameNode.endPosition),
    pointsToRange(node.startPosition, node.endPosition),
    typeNode !== null ? typeNode.text : ""
  );
  walker.fileItem.items.push(propertyItem);
  node.children.forEach((e1) => {
    if (!e1.type.startsWith("methodmap_property_")) {
      return;
    }
    e1.children.forEach((e2) => {
      // TODO: Property methods do not have parameters and variables
      switch (e2.type) {
        case "methodmap_property_getter":
          readFunctionAndMethod(walker, e2, propertyItem, "get");
          break;
        case "methodmap_property_setter":
          readFunctionAndMethod(walker, e2, propertyItem, "set");
          break;
        default:
          break;
      }
    });
  });
}
