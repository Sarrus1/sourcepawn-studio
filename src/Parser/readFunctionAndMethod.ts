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
} from "./interfaces";
import { parsedLocToRange } from "./utils";
import { processDocStringComment } from "./processComment";
import { addVariableItem } from "./addVariableItem";
import { EnumStructItem } from "../Backend/Items/spEnumStructItem";
import { globalIdentifier, globalItem } from "../Misc/spConstants";
import { ConstantItem } from "../Backend/Items/spConstantItem";
import { MethodItem } from "../Backend/Items/spMethodItem";
import { MethodMapItem } from "../Backend/Items/spMethodmapItem";

export function readFunctionAndMethod(
  parserArgs: spParserArgs,
  accessModifiers: string[] | null,
  returnType: ParsedID | null,
  id: ParsedID,
  loc: ParserLocation,
  docstring: ParsedComment,
  params: ParsedParam[] | null,
  body: FunctionBody | null,
  parent: EnumStructItem | ConstantItem = globalItem
): void {
  // Don't add the float native.
  if (id.id === "float") {
    return;
  }
  const range = parsedLocToRange(id.loc, parserArgs);
  const fullRange = parsedLocToRange(loc, parserArgs);
  const { doc, dep } = processDocStringComment(docstring);
  const { processedParams, details } = processFunctionParams(params);
  const processedReturnType = returnType && returnType.id ? returnType.id : "";
  let item: FunctionItem | MethodItem;
  let key: string = id.id;
  if (parent.name !== globalIdentifier) {
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
    key += parent.name;
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

  if (body === null) {
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
  grandParent: EnumStructItem | MethodMapItem | ConstantItem
) {
  if (!obj) {
    return;
  }
  let declarators: VariableDeclarator[],
    variableType: string,
    doc = "",
    found = false;
  if (obj["type"] === "ForLoopVariableDeclaration") {
    declarators = obj["declarations"];
    variableType = (obj["variableType"] as ParsedID).id;
    found = true;
  }
  if (obj["type"] === "LocalVariableDeclaration") {
    declarators = obj.content.declarations;
    variableType = obj.content.variableType ? obj.content.variableType.id : "";
    doc = obj.content.doc;
    found = true;
  }
  if (found) {
    declarators.forEach((e) => {
      const range = parsedLocToRange(e.id.loc, parserArgs);
      addVariableItem(
        parserArgs,
        e.id.id,
        variableType,
        range,
        parent,
        doc,
        e.id.id + parent.name + grandParent.name
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
  grandParent: EnumStructItem | MethodMapItem | ConstantItem
): void {
  if (!params) {
    return;
  }

  params.forEach((e) => {
    let processedDeclType = "";
    if (typeof e.declarationType === "string") {
      processedDeclType = e.declarationType + " ";
    } else if (Array.isArray(e.declarationType)) {
      processedDeclType = e.declarationType.join(" ") + " ";
    }
    addVariableItem(
      parserArgs,
      e.id.id,
      processedDeclType,
      parsedLocToRange(e.id.loc, parserArgs),
      parent,
      "",
      e.id.id + parent.name + grandParent.name
    );
  });
}
