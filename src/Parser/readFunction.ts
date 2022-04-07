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
import { addVariableItem } from "./addVariableItem2";

export function readFunction(
  parserArgs: spParserArgs,
  accessModifiers: string[] | null,
  returnType: ParsedID | null,
  id: ParsedID,
  loc: ParserLocation,
  docstring: (string | PreprocessorStatement)[] | undefined,
  params: ParsedParam[] | null,
  body: FunctionBody | null
): void {
  const range = parsedLocToRange(id.loc);
  const fullRange = parsedLocToRange(loc);
  const { doc, dep } = processDocStringComment(docstring);
  const { processedParams, details } = processFunctionParams(params);
  const processedReturnType = returnType && returnType.id ? returnType.id : "";
  const functionItem = new FunctionItem(
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
  parserArgs.fileItems.set(id.id, functionItem);
  addParamsAsVariables(parserArgs, params, functionItem);

  if (body === null) {
    // We are in a native or forward.
    return;
  }
  searchVariablesInBody(parserArgs, functionItem, body["body"]);
  return;
}

function searchVariablesInBody(
  parserArgs: spParserArgs,
  parent: FunctionItem,
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
  parent: FunctionItem,
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
  parent: FunctionItem
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
