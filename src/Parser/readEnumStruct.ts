import { SyntaxNode } from "web-tree-sitter";

import { pointsToRange } from "./utils";
import { EnumStructItem } from "../Backend/Items/spEnumStructItem";
import { TreeWalker } from "./spParser";
import { findDoc } from "./readDocumentation";
import { readFunctionAndMethod } from "./readFunctionAndMethod";
import { VariableItem } from "../Backend/Items/spVariableItem";

/**
 * Process an enum struct declaration.
 * @param  {TreeWalker} walker    TreeWalker object.
 * @param  {SyntaxNode} node      Node to process.
 * @returns void
 */
export function readEnumStruct(walker: TreeWalker, node: SyntaxNode): void {
  const nameNode = node.childForFieldName("name");
  const { doc } = findDoc(walker, node);
  const enumStructItem = new EnumStructItem(
    nameNode.text,
    walker.filePath,
    doc,
    pointsToRange(nameNode.startPosition, nameNode.endPosition),
    pointsToRange(node.startPosition, node.endPosition)
  );
  walker.fileItem.items.push(enumStructItem);
  readEnumstructMembers(walker, enumStructItem, node);
}

/**
 * Process the body of an enum struct.
 * @param  {TreeWalker} walker              TreeWalker object.
 * @param  {EnumStructItem} enumstructItem  Parent item of the member.
 * @param  {SyntaxNode} node                Node to process.
 * @returns void
 */
function readEnumstructMembers(
  walker: TreeWalker,
  enumstructItem: EnumStructItem,
  node: SyntaxNode
): void {
  node.children.forEach((e) => {
    switch (e.type) {
      case "enum_struct_field":
        readEnumStructField(walker, e, enumstructItem);
        break;
      case "enum_struct_method":
        readFunctionAndMethod(walker, e, enumstructItem);
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

/**
 * Process an enum struct's field.
 * @param  {TreeWalker} walker                Walker object.
 * @param  {SyntaxNode} node                  Field node.
 * @param  {EnumStructItem} enumStructItem    Parent item of the field.
 * @returns void
 */
function readEnumStructField(
  walker: TreeWalker,
  node: SyntaxNode,
  enumStructItem: EnumStructItem
): void {
  const nameNode = node.childForFieldName("name");
  const typeNode = node.childForFieldName("type");
  const item = new VariableItem(
    nameNode.text,
    walker.filePath,
    enumStructItem,
    pointsToRange(nameNode.startPosition, nameNode.endPosition),
    typeNode.text,
    `${typeNode.text} ${nameNode.text}`,
    "",
    []
  );
  walker.fileItem.items.push(item);
}
