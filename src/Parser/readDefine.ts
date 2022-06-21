import { SyntaxNode } from "web-tree-sitter";

import { DefineItem } from "../Backend/Items/spDefineItem";
import { pointsToRange } from "./utils";
import { TreeWalker } from "./spParser";
import { findDoc } from "./readDocumentation";

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

  const { value, desc } = explodeDefine(valueNode?.text);

  const defineItem = new DefineItem(
    nameNode.text,
    value,
    desc,
    walker.filePath,
    range,
    fullRange,
    dep
  );
  walker.fileItem.items.push(defineItem);
}

function explodeDefine(value: string): { value: string; desc: string } {
  if (!value) {
    return { value: "", desc: "" };
  }
  let match = value.match(/(?:\/\*)(.+?(?=\*\/))/g);
  if (!match) {
    match = value.match(/(?:\/\/)(.*)/);
    if (!match) {
      return { value: "", desc: "" };
    }
    let desc = match[match.length - 1].trim();
    return {
      value: value.slice(0, value.length - desc.length - 2).trim(),
      desc,
    };
  }
  let desc = match[match.length - 1].slice(2).trim();
  return { value: value.slice(0, value.length - desc.length - 4).trim(), desc };
}
