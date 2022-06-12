import * as TreeSitter from "web-tree-sitter";

import { pointsToRange } from "./utils";
import { TreeWalker } from "./spParser";
import { findDoc } from "./readDocumentation";
import { readFunctionAndMethod } from "./readFunctionAndMethod";
import { MethodMapItem } from "../Backend/Items/spMethodmapItem";
import { readProperty } from "./readProperty";

/**
 * Process a methodmap declaration.
 * @param  {TreeWalker} walker            TreeWalker object.
 * @param  {TreeSitter.SyntaxNode} node   Node to process.
 * @returns void
 */
export function readMethodmap(
  walker: TreeWalker,
  node: TreeSitter.SyntaxNode
): void {
  const nameNode = node.childForFieldName("name");
  const inheritNode = node.childForFieldName("inherits");
  const { doc, dep } = findDoc(walker, node);
  const methodmapItem = new MethodMapItem(
    nameNode.text,
    inheritNode?.text,
    doc,
    walker.filePath,
    pointsToRange(nameNode.startPosition, nameNode.endPosition),
    pointsToRange(node.startPosition, node.endPosition),
    walker.isBuiltin
  );
  walker.fileItem.items.push(methodmapItem);
  readMethodmapMembers(walker, methodmapItem, node);
}

/**
 * Process the body of a methodmap.
 * @param  {TreeWalker} walker            TreeWalker object.
 * @param  {EnumStructItem} parent        Parent item of the member.
 * @param  {TreeSitter.SyntaxNode} node   Node to process.
 * @returns void
 */
function readMethodmapMembers(
  walker: TreeWalker,
  parent: MethodMapItem,
  node: TreeSitter.SyntaxNode
): void {
  node.children.forEach((e) => {
    switch (e.type) {
      case "methodmap_method":
        readFunctionAndMethod(walker, e, parent);
        break;
      case "methodmap_method_constructor":
        readFunctionAndMethod(walker, e, parent);
        break;
      case "methodmap_method_destructor":
        readFunctionAndMethod(walker, e, parent);
        break;
      case "methodmap_native":
        readFunctionAndMethod(walker, e, parent);
        break;
      case "methodmap_native_constructor":
        readFunctionAndMethod(walker, e, parent);
        break;
      case "methodmap_native_destructor":
        readFunctionAndMethod(walker, e, parent);
        break;
      case "methodmap_property":
        readProperty(walker, e, parent);
        break;
      case "comment":
        walker.pushComment(e);
        break;
      case "preproc_pragma_deprecated":
        walker.deprecated.push(e);
        break;
      default:
        break;
    }
  });
}
