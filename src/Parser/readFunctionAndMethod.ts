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
  PreprocessorStatement,
} from "./interfaces";
import { parsedLocToRange } from "./utils";
import { processDocStringComment } from "./processComment";
import { addVariableItem } from "./addVariableItem";
import { EnumStructItem } from "../Backend/Items/spEnumStructItem";
import { globalIdentifier, globalItem } from "../Misc/spConstants";
import { ConstantItem } from "../Backend/Items/spConstantItem";
import { MethodItem } from "../Backend/Items/spMethodItem";

export function readFunctionAndMethod(
  parserArgs: spParserArgs,
  accessModifiers: string[] | null,
  returnType: ParsedID | null,
  id: ParsedID,
  loc: ParserLocation,
  docstring: (string | PreprocessorStatement)[] | undefined,
  params: ParsedParam[] | null,
  body: FunctionBody | null,
  parent: EnumStructItem | ConstantItem = globalItem
): void {
  const range = parsedLocToRange(id.loc);
  const fullRange = parsedLocToRange(loc);
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
  addParamsAsVariables(parserArgs, params, item);

  if (body === null) {
    // We are in a native or forward.
    return;
  }
  searchVariablesInBody(parserArgs, item, body["body"]);
  return;
}

function searchVariablesInBody(
  parserArgs: spParserArgs,
  parent: FunctionItem | MethodItem,
  arr
) {
  if (!arr) {
    return;
  }
  if (Array.isArray(arr)) {
    for (let obj of arr) {
      recursiveVariableSearch(parserArgs, parent, obj);
    }
  } else {
    recursiveVariableSearch(parserArgs, parent, arr);
  }
}

function recursiveVariableSearch(
  parserArgs: spParserArgs,
  parent: FunctionItem | MethodItem,
  obj
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
    variableType = "int";
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
      const range = parsedLocToRange(e.id.loc);
      addVariableItem(
        parserArgs,
        e.id.id,
        variableType,
        range,
        parent,
        doc,
        e.id.id + parent.name
      );
    });
    return;
  }
  for (let k in obj) {
    if (typeof obj[k] == "object" && obj[k] !== null) {
      searchVariablesInBody(parserArgs, parent, obj[k]);
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
  parent: FunctionItem | MethodItem
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
      parsedLocToRange(e.id.loc),
      parent,
      "",
      e.id.id + parent.name
    );
  });
}
