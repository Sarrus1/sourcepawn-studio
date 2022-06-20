import { CompletionItemKind, Range } from "vscode";
import * as TreeSitter from "web-tree-sitter";

import { EnumItem } from "../Backend/Items/spEnumItem";
import { EnumMemberItem } from "../Backend/Items/spEnumMemberItem";
import { pointsToRange } from "./utils";
import { TreeWalker } from "./spParser";
import { commentToDoc, findDoc } from "./readDocumentation";

/**
 * Process an enum declaration.
 * @param  {TreeWalker} walker            TreeWalker object.
 * @param  {TreeSitter.SyntaxNode} node   Node to process.
 * @returns void
 */
export function readEnum(
  walker: TreeWalker,
  node: TreeSitter.SyntaxNode
): void {
  const { name, nameRange } = getEnumNameAndRange(walker, node);
  // FIXME: argument_declarations contain () as well. This is not specified in node-types.json
  const { doc, dep } = findDoc(walker, node);
  const enumItem = new EnumItem(
    name,
    walker.filePath,
    doc,
    nameRange,
    pointsToRange(node.startPosition, node.endPosition)
  );
  walker.fileItem.items.push(enumItem);
  readEnumMembers(
    walker,
    node.children.find((e) => e.type === "enum_entries"),
    enumItem
  );
}

/**
 * Generate the name and the range of a potential anonymous enum.
 * @param  {TreeWalker} walker            TreeWalker object.
 * @param  {TreeSitter.SyntaxNode} node   Node to process.
 * @returns void
 */
function getEnumNameAndRange(
  walker: TreeWalker,
  node: TreeSitter.SyntaxNode
): { name: string; nameRange: Range } {
  const nameNode = node.childForFieldName("name");
  if (nameNode) {
    return {
      name: nameNode.text,
      nameRange: pointsToRange(nameNode.startPosition, nameNode.endPosition),
    };
  }
  walker.anonEnumCount++;
  const name = `Enum#${walker.anonEnumCount}`;
  const nameRange = new Range(
    node.startPosition.row,
    0,
    node.startPosition.row,
    6
  );
  return { name, nameRange };
}

/**
 * Process the body of an enum.
 * @param  {TreeWalker} walker            TreeWalker object.
 * @param  {TreeSitter.SyntaxNode} body   Body to process.
 * @param  {EnumItem} enumItem            Parent enum of the body.
 * @returns void
 */
function readEnumMembers(
  walker: TreeWalker,
  body: TreeSitter.SyntaxNode,
  enumItem: EnumItem
): void {
  if (!body) {
    return;
  }
  body.children.forEach((child) => {
    const prevEnumMember =
      walker.fileItem.items[walker.fileItem.items.length - 1];
    if (
      child.type === "comment" &&
      prevEnumMember?.kind === CompletionItemKind.EnumMember
    ) {
      prevEnumMember.description += commentToDoc(child.text);
    }
    if (child.type !== "enum_entry") {
      return;
    }
    const entry = child.childForFieldName("name");
    const range = pointsToRange(entry.startPosition, entry.endPosition);
    const memberItem = new EnumMemberItem(
      entry.text,
      walker.filePath,
      range,
      enumItem
    );
    walker.fileItem.items.push(memberItem);
  });
}
