import * as TreeSitter from "web-tree-sitter";

import { spParserArgs } from "./interfaces";
import { FunctionItem } from "../Backend/Items/spFunctionItem";
import {
  FormalParameter,
  FunctionParam,
  VariableDeclarator,
  VariableDeclaration,
} from "./interfaces";
import { parsedLocToRange, pointsToRange } from "./utils";
import { addVariableItem } from "./addVariableItem";
import { EnumStructItem } from "../Backend/Items/spEnumStructItem";
import { globalItem } from "../Misc/spConstants";
import { ConstantItem } from "../Backend/Items/spConstantItem";
import { MethodItem } from "../Backend/Items/spMethodItem";
import { PropertyItem } from "../Backend/Items/spPropertyItem";
import { CompletionItemKind } from "vscode";
import { TreeWalker } from "./spParser";
import { spLangObj } from "../spIndex";

//TODO: Add typing.
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
  item = new FunctionItem(
    nameNode.text,
    node.text,
    "doc",
    undefined,
    walker.filePath,
    walker.isBuiltin,
    pointsToRange(nameNode.startPosition, nameNode.endPosition),
    returnTypeNode ? returnTypeNode.text : "",
    pointsToRange(node.startPosition, node.endPosition),
    undefined,
    storageClassNode ? [storageClassNode.text] : [],
    undefined
  );
  readBodyVariables(
    walker,
    node.children.find((e) => e.type === "block"),
    item
  );
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
  // FIXME: This query does not work.
  const query = spLangObj.query("(block (variable_declaration_statement))");
  const res = query.matches(body);
  for (let node of body.children) {
    // let declarators: VariableDeclarator[],
    //   variableType: string,
    //   doc = "",
    //   found = false,
    //   processedDeclType = "",
    //   modifier = "";
    // if (e.type === "ForLoopVariableDeclaration") {
    //   declarators = e["declarations"];
    //   variableType = "int ";
    //   found = true;
    // } else if (e["type"] === "LocalVariableDeclaration") {
    //   const content: VariableDeclaration = e.content;
    //   declarators = content.declarations;
    //   if (content.variableType) {
    //     variableType = content.variableType.name.id;
    //     modifier = content.variableType.modifier || "";
    //   }
    //   //doc = content.doc;
    //   if (content.accessModifiers !== null) {
    //     processedDeclType = content.accessModifiers.join(" ");
    //   }
    //   found = true;
    // }
    // if (found) {
    //   for (let decl of declarators) {
    //     const range = parsedLocToRange(decl.id.loc, parserArgs);
    //     // Break if the item's range is not in the fullrange of the parent.
    //     // This means it belongs to another method of the same enum struct/methodmap.
    //     // We can assume that all the next items will be the same.
    //     if (!parent.fullRange.contains(range)) {
    //       break;
    //     }
    //     const arrayInitialer = decl.arrayInitialer || "";
    //     variableType = variableType || "";
    //     addVariableItem(
    //       parserArgs,
    //       decl.id.id,
    //       variableType,
    //       range,
    //       parent,
    //       doc,
    //       `${processedDeclType}${variableType}${modifier}${
    //         decl.id.id
    //       }${arrayInitialer.trim()};`.trim()
    //     );
    //   }
    // }
  }
}

function processFunctionParams(
  params: FormalParameter[] | null,
  doc: string | undefined
): FunctionParam[] {
  if (!params) {
    return [];
  }
  const processedParams = params.map((e) => {
    let documentation = "";
    if (doc) {
      const match = doc.match(
        new RegExp(`@param\\s+(?:\\b${e.id.id}\\b)([^\\@]+)`)
      );
      if (match) {
        documentation = match[1].replace(/\*/gm, "").trim();
      }
    }
    return {
      label: e.id.id,
      documentation,
    } as FunctionParam;
  });
  return processedParams;
}

function addParamsAsVariables(
  parserArgs: spParserArgs,
  params: FormalParameter[] | null,
  parent: FunctionItem | MethodItem,
  processedParams: FunctionParam[]
): void {
  if (!params) {
    return;
  }

  params.forEach((param) => {
    let processedDeclType = "";
    if (typeof param.declarationType === "string") {
      processedDeclType = param.declarationType;
    } else if (Array.isArray(param.declarationType)) {
      processedDeclType = param.declarationType.join(" ");
    }
    const type =
      param.parameterType && param.parameterType.name
        ? param.parameterType.name.id
        : "";
    const modifiers = param.parameterType ? param.parameterType.modifier : "";
    const doc = processedParams.find((e) => e.label === param.id.id);
    addVariableItem(
      parserArgs,
      param.id.id,
      type,
      parsedLocToRange(param.id.loc, parserArgs),
      parent,
      doc ? doc.documentation : "",
      `${processedDeclType} ${type}${modifiers}${param.id.id};`
    );
  });
}
