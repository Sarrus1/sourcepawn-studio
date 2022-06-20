import { CompletionItemKind, Range } from "vscode";
import { SyntaxNode } from "web-tree-sitter";

import { FunctionItem } from "../Backend/Items/spFunctionItem";
import { pointsToRange } from "./utils";
import { EnumStructItem } from "../Backend/Items/spEnumStructItem";
import { globalItem } from "../Misc/spConstants";
import { ConstantItem } from "../Backend/Items/spConstantItem";
import { MethodItem } from "../Backend/Items/spMethodItem";
import { PropertyItem } from "../Backend/Items/spPropertyItem";
import { TreeWalker } from "./spParser";
import { variableQuery } from "../spIndex";
import { readVariable } from "./readVariable";
import { VariableItem } from "../Backend/Items/spVariableItem";
import { findDoc } from "./readDocumentation";
import { MethodMapItem } from "../Backend/Items/spMethodmapItem";

export type MethodParent = EnumStructItem | PropertyItem | MethodMapItem;
const MmEs = [CompletionItemKind.Struct, CompletionItemKind.Class];

/**
 * @param  {TreeWalker} walker                                TreeWalker object.
 * @param  {SyntaxNode} node                                  Node to process.
 * @param  {EnumStructItem|PropertyItem|ConstantItem} parent  Parent of the method. Defaults to globalItem.
 * @returns void
 */
export function readFunctionAndMethod(
  walker: TreeWalker,
  node: SyntaxNode,
  parent: MethodParent | ConstantItem = globalItem,
  getSet?: string
): void {
  let item: FunctionItem | MethodItem;
  const nameNode = node.childForFieldName("name");
  const returnTypeNode = node.childForFieldName("returnType");
  const storageClassNode = node.children.find(
    (e) => e.type === "function_storage_class"
  );
  const params = node.children.find((e) => e.type === "argument_declarations");
  const { doc, dep } = findDoc(walker, node);
  const returnType = returnTypeNode ? returnTypeNode.text : "";
  const storageClass = storageClassNode ? [storageClassNode.text] : [];
  const functionTypeNode = node.children.find(
    (e) => e.type === "function_definition_type"
  );
  const functionType = functionTypeNode ? functionTypeNode.text : "";
  // TODO: Separate storage classes and function types.
  storageClass.push(functionType);
  if (parent === globalItem) {
    item = new FunctionItem(
      nameNode.text,
      `${storageClass.join(" ")} ${returnType} ${nameNode.text}${
        params.text
      }`.trim(),
      doc,
      walker.filePath,
      pointsToRange(nameNode.startPosition, nameNode.endPosition),
      returnType,
      pointsToRange(node.startPosition, node.endPosition),
      dep,
      storageClass
    );
  } else if (getSet) {
    // Handle properties' getters/setters.
    const idx = node.text.search(getSet);
    const nameRange = new Range(
      node.startPosition.row,
      node.startPosition.column + idx,
      node.startPosition.row,
      node.startPosition.column + idx + "get".length
    );
    item = new MethodItem(
      parent as PropertyItem,
      getSet,
      `${storageClass.join(" ")} ${returnType} ${getSet}${"()"}`.trim(),
      doc,
      returnType,
      walker.filePath,
      nameRange,
      pointsToRange(node.startPosition, node.endPosition),
      dep
    );
  } else {
    item = new MethodItem(
      parent as MethodParent,
      nameNode.text,
      `${storageClass.join(" ")} ${returnType} ${nameNode.text}${
        params.text
      }`.trim(),
      doc,
      returnType,
      walker.filePath,
      pointsToRange(nameNode.startPosition, nameNode.endPosition),
      pointsToRange(node.startPosition, node.endPosition),
      dep
    );
  }
  readBodyVariables(
    walker,
    node.children.find((e) => e.type === "block"),
    item
  );
  addParamsAsVariables(walker, params, item, doc);
  walker.fileItem.items.push(item);
}

/**
 * Process the variables of a function/method's body.
 * @param  {TreeWalker} walker                TreeWalker object.
 * @param  {SyntaxNode} node                  Node to process.
 * @param  {FunctionItem|MethodItem} parent   Parent item of the body.
 * @returns void
 */
export function readBodyVariables(
  walker: TreeWalker,
  body: SyntaxNode,
  parent: FunctionItem | MethodItem
): void {
  if (body === undefined) {
    return;
  }
  const res = variableQuery.captures(body);
  res.forEach((capture) => {
    readVariable(walker, capture.node, parent);
  });
}

/**
 * Process the params of a function/method and adds them as variables.
 * @param  {TreeWalker} walker                TreeWalker object.
 * @param  {SyntaxNode} params                Params node.
 * @param  {FunctionItem|MethodItem} parent   Parent function/method item of the params.
 * @param  {string|undefined} doc             Documentation of the function if it exists.
 * @returns void
 */
function addParamsAsVariables(
  walker: TreeWalker,
  params: SyntaxNode,
  parent: FunctionItem | MethodItem,
  doc: string | undefined
): void {
  if (!params) {
    return;
  }

  for (const param of params.children) {
    if (param.type !== "argument_declaration") {
      continue;
    }
    const variableTypeNode = param.childForFieldName("type");
    const variableType = variableTypeNode ? variableTypeNode.text : "";
    const variableNameNode = param.childForFieldName("name");
    const storageClass = [];
    if (param.children.find((e) => e.text === "const")) {
      storageClass.push("const");
    }
    let documentation = "";
    if (doc) {
      const match = doc.match(
        new RegExp(`@param\\s+(?:\\b${variableNameNode.text}\\b)([^\\@]+)`)
      );
      if (match) {
        documentation = match[1].replace(/\*/gm, "").trim();
      }
    }

    const variableItem = new VariableItem(
      variableNameNode.text,
      walker.filePath,
      parent,
      pointsToRange(
        variableNameNode.startPosition,
        variableNameNode.endPosition
      ),
      variableType,
      `${storageClass.join(" ")} ${variableType} ${variableNameNode.text}`,
      documentation,
      storageClass
    );
    walker.fileItem.items.push(variableItem);
    parent.params.push(variableItem);
  }
}
