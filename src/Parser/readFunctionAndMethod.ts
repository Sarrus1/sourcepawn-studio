import { spParserArgs } from "./spParser";
import { FunctionItem } from "../Backend/Items/spFunctionItem";
import {
  ParsedParam,
  ParsedID,
  ParserLocation,
  FunctionParam,
  FunctionBody,
  VariableDeclarator,
  ParsedComment,
  VariableDeclaration,
} from "./interfaces";
import { parsedLocToRange } from "./utils";
import { processDocStringComment } from "./processComment";
import { addVariableItem } from "./addVariableItem";
import { EnumStructItem } from "../Backend/Items/spEnumStructItem";
import { globalItem } from "../Misc/spConstants";
import { ConstantItem } from "../Backend/Items/spConstantItem";
import { MethodItem } from "../Backend/Items/spMethodItem";
import { MethodMapItem } from "../Backend/Items/spMethodmapItem";
import { PropertyItem } from "../Backend/Items/spPropertyItem";
import { CompletionItemKind } from "vscode";

export function readFunctionAndMethod(
  parserArgs: spParserArgs,
  accessModifiers: string[] | null,
  returnType: ParsedID | null,
  id: ParsedID,
  loc: ParserLocation,
  docstring: ParsedComment,
  params: ParsedParam[] | null,
  body: FunctionBody | null,
  txt: string,
  parent: EnumStructItem | PropertyItem | ConstantItem = globalItem
): void {
  const MmEs = [CompletionItemKind.Struct, CompletionItemKind.Class];

  txt = txt.replace(/\s*\r?\n\s*/gm, " ").trim();
  if (id.id === "foo") {
    console.debug("rt");
  }
  const range = parsedLocToRange(id.loc, parserArgs);
  const fullRange = parsedLocToRange(loc, parserArgs);
  const { doc, dep } = processDocStringComment(docstring);
  const processedParams = processFunctionParams(params, doc);
  let item: FunctionItem | MethodItem;
  let key = id.id;
  if (parent.kind === CompletionItemKind.Property) {
    item = new MethodItem(
      parent as PropertyItem,
      id.id,
      txt,
      doc,
      processedParams,
      returnType ? returnType.id : "",
      parserArgs.filePath,
      range,
      parserArgs.IsBuiltIn,
      fullRange,
      dep
    );
    key += `-${parent.name}-${(parent as PropertyItem).parent.name}`;
  } else if (MmEs.includes(parent.kind)) {
    item = new MethodItem(
      parent as EnumStructItem,
      id.id,
      txt,
      doc,
      processedParams,
      returnType ? returnType.id : "",
      parserArgs.filePath,
      range,
      parserArgs.IsBuiltIn,
      fullRange,
      dep
    );
    key += `-${parent.name}`;
  } else {
    item = new FunctionItem(
      id.id,
      txt,
      doc,
      processedParams,
      parserArgs.filePath,
      parserArgs.IsBuiltIn,
      range,
      returnType ? returnType.id : "",
      fullRange,
      dep,
      accessModifiers
    );
  }
  parserArgs.fileItems.set(key, item);
  addParamsAsVariables(parserArgs, params, item, parent, processedParams);

  if (!body) {
    // We are in a native or forward.
    return;
  }
  readBodyVariables(parserArgs, item, parent);
  return;
}

function readBodyVariables(
  parserArgs: spParserArgs,
  parent: FunctionItem | MethodItem,
  grandParent: EnumStructItem | MethodMapItem | ConstantItem
) {
  for (let e of parserArgs.variableDecl) {
    let declarators: VariableDeclarator[],
      variableType: string,
      doc = "",
      found = false,
      processedDeclType = "",
      modifier = "";

    if (e.type === "ForLoopVariableDeclaration") {
      declarators = e["declarations"];
      variableType = "int ";
      found = true;
    } else if (e["type"] === "LocalVariableDeclaration") {
      const content: VariableDeclaration = e.content;
      declarators = content.declarations;
      if (content.variableType) {
        variableType = content.variableType.name.id;
        modifier = content.variableType.modifier || "";
      }
      //doc = content.doc;
      if (typeof content.variableDeclarationType === "string") {
        processedDeclType = content.variableDeclarationType;
      } else if (Array.isArray(content.variableDeclarationType)) {
        processedDeclType = content.variableDeclarationType.join(" ");
      }
      found = true;
    }

    if (found) {
      for (let decl of declarators) {
        const range = parsedLocToRange(decl.id.loc, parserArgs);
        // Break if the item's range is not in the fullrange of the parent.
        // This means it belongs to another method of the same enum struct/methodmap.
        // We can assume that all the next items will be the same.
        if (!parent.fullRange.contains(range)) {
          break;
        }
        const arrayInitialer = decl.arrayInitialer || "";
        variableType = variableType || "";
        addVariableItem(
          parserArgs,
          decl.id.id,
          variableType,
          range,
          parent,
          doc,
          `${processedDeclType}${variableType}${modifier}${
            decl.id.id
          }${arrayInitialer.trim()};`.trim(),
          `${decl.id.id}-${parent.name}-${grandParent.name}-${
            grandParent.kind === CompletionItemKind.Property
              ? (grandParent as PropertyItem).parent.name
              : ""
          }`
        );
      }
    }
  }
}

function processFunctionParams(
  params: ParsedParam[] | null,
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
  params: ParsedParam[] | null,
  parent: FunctionItem | MethodItem,
  grandParent: EnumStructItem | MethodMapItem | PropertyItem | ConstantItem,
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
      `${processedDeclType} ${type}${modifiers}${param.id.id};`,
      `${param.id.id}-${parent.name}-${grandParent.name}-${
        grandParent.kind === CompletionItemKind.Property
          ? (grandParent as PropertyItem).parent.name
          : ""
      }`
    );
  });
}
