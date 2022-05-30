import * as TreeSitter from "web-tree-sitter";

import { pointsToRange } from "./utils";
import { EnumStructItem } from "../Backend/Items/spEnumStructItem";
import { TreeWalker } from "./spParser";
import { findDoc } from "./readDocumentation";
import { readFunctionAndMethod } from "./readFunctionAndMethod";
import { VariableItem } from "../Backend/Items/spVariableItem";

/**
 * Process an enum struct declaration.
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {EnumstructDeclaration} res  Object containing the enum struct declaration details.
 * @returns void
 */
export function readEnumStruct(
  walker: TreeWalker,
  node: TreeSitter.SyntaxNode
): void {
  const nameNode = node.childForFieldName("name");
  const { doc, dep } = findDoc(walker, node);
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
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {EnumStructItem} enumstructItem  The parent of the enum struct members.
 * @param  {EnumstructMemberDeclaration[]} body  The body of the enum struct to parse.
 * @returns void
 */
function readEnumstructMembers(
  walker: TreeWalker,
  enumstructItem: EnumStructItem,
  node: TreeSitter.SyntaxNode
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
      default:
        break;
    }
  });
}

/**
 * Process an enum struct's field.
 * @param  {TreeWalker} walker                Walker object.
 * @param  {TreeSitter.SyntaxNode} node       Field node.
 * @param  {EnumStructItem} enumStructItem    Parent item of the field.
 * @returns void
 */
function readEnumStructField(
  walker: TreeWalker,
  node: TreeSitter.SyntaxNode,
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
