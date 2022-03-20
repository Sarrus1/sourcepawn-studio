import { spParserArgs } from "./spParser";
import { FunctionItem } from "../Backend/Items/spFunctionItem";
import { ParsedFunctionParam, ParsedID, ParserLocation } from "./interfaces";
import { parsedLocToRange } from "./utils";
import { FunctionParam } from "../Backend/Items/spItems";
import { processDocStringComment } from "./processComment";

export function readFunction(
  parserArgs: spParserArgs,
  accessModifier: string[] | null,
  returnType: ParsedID | null,
  id: ParsedID,
  loc: ParserLocation,
  docstring: string[] | undefined,
  params: ParsedFunctionParam[] | null,
  body: any
): void {
  const range = parsedLocToRange(id.loc);
  const fullRange = parsedLocToRange(loc);
  const { doc, dep } = processDocStringComment(docstring);
  const { processedParams, details } = processFunctionParams(params);
  const processedReturnType = returnType && returnType.id ? returnType.id : "";
  const functionItem = new FunctionItem(
    id.id,
    `${processedReturnType} ${id.id}(${details.replace(/, $/, "")})`,
    doc,
    processedParams,
    parserArgs.filePath,
    parserArgs.IsBuiltIn,
    range,
    returnType ? returnType.id : "",
    fullRange,
    dep
  );
  parserArgs.fileItems.set(id.id, functionItem);
  return;
}

interface ProcessedParams {
  processedParams: FunctionParam[];
  details: string;
}

function processFunctionParams(
  params: ParsedFunctionParam[] | null
): ProcessedParams {
  if (params === undefined || params === null) {
    return { processedParams: [], details: "" };
  }
  const processedParams = [];
  let details = "";
  params.forEach((e) => {
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
    let processedType = e.parameterType ? e.parameterType.id + " " : "";
    details += processedDeclType + processedType + e.id.id + ", ";
  });
  return { processedParams, details };
}
