import * as TreeSitter from "web-tree-sitter";

import { DefineItem } from "../Backend/Items/spDefineItem";
import { pointsToRange } from "./utils";
import { TreeWalker } from "./spParser";
import { findDoc } from "./readDocumentation";

/**
 * Process a define statement.
 * @param  {TreeWalker} walker            TreeWalker object.
 * @param  {TreeSitter.SyntaxNode} node   Node to process.
 * @returns void
 */
export function readDefine(
  walker: TreeWalker,
  node: TreeSitter.SyntaxNode
): void {
  const nameNode = node.childForFieldName("name");
  const valueNode = node.childForFieldName("value");
  const range = pointsToRange(nameNode.startPosition, nameNode.endPosition);
  const fullRange = pointsToRange(node.startPosition, node.endPosition);
  const { doc, dep } = findDoc(walker, node);

  const defineItem = new DefineItem(
    nameNode.text,
    valueNode !== null ? valueNode.text.trim() : "",
    "",
    walker.filePath,
    range,
    walker.isBuiltin,
    fullRange,
    dep
  );
  walker.fileItem.items.push(defineItem);
}
