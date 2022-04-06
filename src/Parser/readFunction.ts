import { spParserArgs } from "./spParser";
import { FunctionItem } from "../Backend/Items/spFunctionItem";
import {
  ParsedParam,
  ParsedID,
  ParserLocation,
  ProcessedParams,
  FunctionParam,
  VariableDeclaration,
  FunctionBody,
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
  docstring: string[] | undefined,
  params: ParsedParam[] | null,
  body: FunctionBody
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
  eachRecursive(parserArgs, functionItem, body["body"]);
  return;
}

// This function handles arrays and objects
function eachRecursive(
  parserArgs: spParserArgs,
  parent: FunctionItem,
  arr: any[] | null
) {
  if (!arr) {
    return;
  }
  for (let obj of arr) {
    if (obj["type"] === "LocalVariableDeclaration") {
      const decl = obj["content"] as VariableDeclaration;
      decl.declarations.forEach((e) => {
        const range = parsedLocToRange(e.id.loc);
        addVariableItem(
          parserArgs,
          e.id.id,
          decl.variableType ? decl.variableType.id : "",
          range,
          parent,
          decl.doc,
          e.id.id + parent.name
        );
      });
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
