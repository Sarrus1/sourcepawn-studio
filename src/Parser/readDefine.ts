import { SyntaxNode } from "web-tree-sitter";

import { DefineItem } from "../Backend/Items/spDefineItem";
import { pointsToRange } from "./utils";
import { TreeWalker } from "./spParser";
import { commentToDoc, findDoc } from "./readDocumentation";

/**
 * Process a define statement.
 * @param  {TreeWalker} walker    TreeWalker object.
 * @param  {SyntaxNode} node      Node to process.
 * @returns void
 */
export function readDefine(walker: TreeWalker, node: SyntaxNode): void {
  const nameNode = node.childForFieldName("name");
  const valueNode = node.childForFieldName("value");
  const range = pointsToRange(nameNode.startPosition, nameNode.endPosition);
  const fullRange = pointsToRange(node.startPosition, node.endPosition);
  const { dep } = findDoc(walker, node);
  const doc = node.children.find((e) => e.type === "comment")?.text || "";

  const defineItem = new DefineItem(
    nameNode.text,
    valueNode?.text || "",
    commentToDoc(doc),
    walker.filePath,
    range,
    fullRange,
    dep
  );
  walker.fileItem.items.push(defineItem);
}
