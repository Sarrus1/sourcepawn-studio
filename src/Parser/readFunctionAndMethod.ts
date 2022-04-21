import { spParserArgs } from "./spParser";
import { FunctionItem } from "../Backend/Items/spFunctionItem";
import {
  ParsedParam,
  ParsedID,
  ParserLocation,
  ProcessedParams,
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
  parent: EnumStructItem | PropertyItem | ConstantItem = globalItem
): void {
  const MmEs = [CompletionItemKind.Struct, CompletionItemKind.Class];

  const range = parsedLocToRange(id.loc, parserArgs);
  const fullRange = parsedLocToRange(loc, parserArgs);
  const { doc, dep } = processDocStringComment(docstring);
  const { processedParams, details } = processFunctionParams(params);
  const processedReturnType = returnType && returnType.id ? returnType.id : "";
  let item: FunctionItem | MethodItem;
  let key: string = id.id;
  if (parent.kind === CompletionItemKind.Property) {
    item = new MethodItem(
      parent as PropertyItem,
      id.id,
      `${processedReturnType} ${id.id}(${details.replace(/, $/, "")})`.trim(),
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
      `${processedReturnType} ${id.id}(${details.replace(/, $/, "")})`.trim(),
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
      `${processedReturnType} ${id.id}(${details.replace(/, $/, "")})`.trim(),
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
  addParamsAsVariables(parserArgs, params, item, parent);

  if (!body) {
    // We are in a native or forward.
    return;
  }
  searchVariablesInBody(parserArgs, body["body"], item, parent);
  return;
}

function searchVariablesInBody(
  parserArgs: spParserArgs,
  arr,
  parent: FunctionItem | MethodItem,
  grandParent: EnumStructItem | MethodMapItem | ConstantItem
) {
  if (!arr) {
    return;
  }
  if (Array.isArray(arr)) {
    for (let obj of arr) {
      recursiveVariableSearch(parserArgs, obj, parent, grandParent);
    }
  } else {
    recursiveVariableSearch(parserArgs, arr, parent, grandParent);
  }
}

function recursiveVariableSearch(
  parserArgs: spParserArgs,
  obj,
  parent: FunctionItem | MethodItem,
  grandParent: EnumStructItem | MethodMapItem | PropertyItem | ConstantItem
) {
  if (!obj) {
    return;
  }
  let declarators: VariableDeclarator[],
    variableType: string,
    doc = "",
    found = false,
    processedDeclType = "",
    modifier = "";

  if (obj.type === "ForLoopVariableDeclaration") {
    declarators = obj["declarations"];
    variableType = "int ";
    found = true;
  } else if (obj["type"] === "LocalVariableDeclaration") {
    const content: VariableDeclaration = obj.content;
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
    declarators.forEach((e) => {
      const range = parsedLocToRange(e.id.loc, parserArgs);
      const arrayInitialer = e.arrayInitialer || "";
      variableType = variableType || "";
      addVariableItem(
        parserArgs,
        e.id.id,
        variableType,
        range,
        parent,
        doc,
        `${processedDeclType} ${variableType}${modifier}${
          e.id.id
        }${arrayInitialer.trim()};`.trim(),
        `${e.id.id}-${parent.name}-${grandParent.name}-${
          grandParent.kind === CompletionItemKind.Property
            ? (grandParent as PropertyItem).parent.name
            : ""
        }`
      );
    });
    return;
  }
  for (let k in obj) {
    if (typeof obj[k] == "object" && obj[k] !== null) {
      searchVariablesInBody(parserArgs, obj[k], parent, grandParent);
    }
  }
}

function processFunctionParams(params: ParsedParam[] | null): ProcessedParams {
  if (params === undefined || params === null) {
    return { processedParams: [], details: "" };
  }
  const processedParams = [];
  let details = "";
  params.forEach((e) => {
    // Handle "..." tokens.
    const param: FunctionParam = {
      label: e.id.id,
      documentation: "",
    };
    processedParams.push(param);
    let processedDeclType = "";
    if (typeof e.declarationType === "string") {
      processedDeclType = e.declarationType + " ";
    } else if (Array.isArray(e.declarationType)) {
      processedDeclType = e.declarationType.join(" ") + " ";
    }
    const processedType =
      e.parameterType && e.parameterType.name
        ? e.parameterType.name.id + e.parameterType.modifier
        : "";
    details += processedDeclType + processedType + e.id.id + ", ";
  });
  return { processedParams, details };
}

function addParamsAsVariables(
  parserArgs: spParserArgs,
  params: ParsedParam[] | null,
  parent: FunctionItem | MethodItem,
  grandParent: EnumStructItem | MethodMapItem | PropertyItem | ConstantItem
): void {
  if (!params) {
    return;
  }

  params.forEach((e) => {
    let processedDeclType = "";
    if (typeof e.declarationType === "string") {
      processedDeclType = e.declarationType;
    } else if (Array.isArray(e.declarationType)) {
      processedDeclType = e.declarationType.join(" ");
    }
    const type =
      e.parameterType && e.parameterType.name ? e.parameterType.name.id : "";
    const modifiers = e.parameterType ? e.parameterType.modifier : "";
    addVariableItem(
      parserArgs,
      e.id.id,
      type,
      parsedLocToRange(e.id.loc, parserArgs),
      parent,
      "",
      `${processedDeclType} ${type}${modifiers}${e.id.id};`,
      `${e.id.id}-${parent.name}-${grandParent.name}-${
        grandParent.kind === CompletionItemKind.Property
          ? (grandParent as PropertyItem).parent.name
          : ""
      }`
    );
  });
}
