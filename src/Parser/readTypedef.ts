import { SyntaxNode } from "web-tree-sitter";

import { TypedefItem } from "../Backend/Items/spTypedefItem";
import { pointsToRange } from "./utils";
import { TreeWalker } from "./spParser";
import { findDoc } from "./readDocumentation";

/**
 * Process a typedef declaration.
 * @param  {TreeWalker} walker    TreeWalker object.
 * @param  {SyntaxNode} node      Node to process.
 * @returns void
 */
export function readTypedef(walker: TreeWalker, node: SyntaxNode): void {
  const nameNode = node.childForFieldName("name");
  const body = node.children.find((e) => e.type === "typedef_expression");
  const typeNode = body.childForFieldName("returnType");
  const { doc, dep } = findDoc(walker, node);
  const typeDefItem = new TypedefItem(
    nameNode.text,
    node.text,
    walker.filePath,
    doc,
    typeNode.text,
    pointsToRange(nameNode.startPosition, nameNode.endPosition),
    pointsToRange(node.startPosition, node.endPosition),
    body.children
      .find((e) => e.type === "argument_declarations")
      .children.filter((e) => e.type === "argument_declaration")
  );
  walker.fileItem.items.push(typeDefItem);
  return;
}
