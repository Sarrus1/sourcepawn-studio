import { spParserArgs } from "./spParser";
import { FunctionItem } from "../Backend/Items/spFunctionItem";
import {
  ParsedParam,
  ParsedID,
  ParserLocation,
  ProcessedParams,
  FunctionParam,
} from "./interfaces";
import { parsedLocToRange } from "./utils";
import { processDocStringComment } from "./processComment";

export function readFunction(
  parserArgs: spParserArgs,
  accessModifiers: string[] | null,
  returnType: ParsedID | null,
  id: ParsedID,
  loc: ParserLocation,
  docstring: string[] | undefined,
  params: ParsedParam[] | null,
  body: any
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
  return;
}

function processFunctionParams(params: ParsedParam[] | null): ProcessedParams {
  if (params === undefined || params === null) {
    return { processedParams: [], details: "" };
  }
  const processedParams = [];
  let details = "";
  params.forEach((e) => {
    // Handle "..." tokens.
    const id = e.id === "..." ? "..." : e.id.id;
    const param: FunctionParam = {
      label: id,
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
    details += processedDeclType + processedType + id + ", ";
  });
  return { processedParams, details };
}
