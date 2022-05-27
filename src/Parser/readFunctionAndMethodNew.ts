import * as TreeSitter from "web-tree-sitter";

import { FunctionItem } from "../Backend/Items/spFunctionItem";
import { FunctionParam } from "./interfaces";
import { pointsToRange } from "./utils";
import { EnumStructItem } from "../Backend/Items/spEnumStructItem";
import { globalItem } from "../Misc/spConstants";
import { ConstantItem } from "../Backend/Items/spConstantItem";
import { MethodItem } from "../Backend/Items/spMethodItem";
import { PropertyItem } from "../Backend/Items/spPropertyItem";
import { CompletionItemKind } from "vscode";
import { TreeWalker } from "./spParser";
import { spLangObj } from "../spIndex";
import { readVariable } from "./readVariable";
import { VariableItem } from "../Backend/Items/spVariableItem";
import { findDocumentation } from "./findDocumentation";

export function readFunctionAndMethod(
  walker: TreeWalker,
  node: TreeSitter.SyntaxNode,
  parent: EnumStructItem | PropertyItem | ConstantItem = globalItem
): void {
  const MmEs = [CompletionItemKind.Struct, CompletionItemKind.Class];
  let item: FunctionItem;
  let nameNode = node.childForFieldName("name");
  let returnTypeNode = node.childForFieldName("returnType");
  let storageClassNode = node.children.find(
    (e) => e.type === "function_storage_class"
  );
  // FIXME: argument_declarations contain () as well. This is not specified in node-types.json
  let params = node.children.find((e) => e.type === "argument_declarations");
  let { doc, dep } = findDocumentation(walker, node, false);
  const processedParams = processFunctionParams(params, doc);
  let returnType = returnTypeNode ? returnTypeNode.text : "";
  let storageClass = storageClassNode ? [storageClassNode.text] : [];
  item = new FunctionItem(
    nameNode.text,
    `${storageClass} ${returnType} ${nameNode.text}${params.text}`.trim(),
    doc,
    processedParams,
    walker.filePath,
    walker.isBuiltin,
    pointsToRange(nameNode.startPosition, nameNode.endPosition),
    returnType,
    pointsToRange(node.startPosition, node.endPosition),
    undefined,
    storageClass,
    undefined
  );
  readBodyVariables(
    walker,
    node.children.find((e) => e.type === "block"),
    item
  );
  addParamsAsVariables(walker, params, item, processedParams);
  walker.fileItem.items.push(item);
}

function readBodyVariables(
  walker: TreeWalker,
  body: TreeSitter.SyntaxNode,
  parent: FunctionItem | MethodItem
) {
  if (body === undefined) {
    return;
  }
  const query = spLangObj.query(
    "(variable_declaration_statement) @declaration.variable"
  );
  const res = query.captures(body);
  res.forEach((capture) => {
    readVariable(walker, capture.node, parent);
  });
}

function processFunctionParams(
  params: TreeSitter.SyntaxNode,
  doc: string | undefined
): FunctionParam[] {
  if (!params) {
    return [];
  }
  const processedParams: FunctionParam[] = [];
  for (let param of params.children) {
    if (param.type !== "argument_declaration") {
      continue;
    }
    const label = param.childForFieldName("name").text;
    let documentation = "";
    if (doc) {
      const match = doc.match(
        new RegExp(`@param\\s+(?:\\b${label}\\b)([^\\@]+)`)
      );
      if (match) {
        documentation = match[1].replace(/\*/gm, "").trim();
      }
    }
    processedParams.push({
      label,
      documentation,
    } as FunctionParam);
  }
  return processedParams;
}

function addParamsAsVariables(
  walker: TreeWalker,
  params: TreeSitter.SyntaxNode,
  parent: FunctionItem | MethodItem,
  processedParams: FunctionParam[]
): void {
  if (!params) {
    return;
  }
  for (let param of params.children) {
    if (param.type !== "argument_declaration") {
      continue;
    }
    const variableTypeNode = param.childForFieldName("type");
    const variableType = variableTypeNode ? variableTypeNode.text : "";
    const variableNameNode = param.childForFieldName("name");
    // FIXME: No storage classes for arguments.
    // This is a problem with Tree sitter.
    const storageClass = [];
    const doc = processedParams.find((e) => e.label === variableNameNode.text);
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
      doc ? doc.documentation : "",
      storageClass
    );
    walker.fileItem.items.push(variableItem);
  }
}
