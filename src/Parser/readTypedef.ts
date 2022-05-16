import {
  FunctagDeclaration,
  spParserArgs,
  TypedefDeclaration,
} from "./interfaces";
import { TypedefItem } from "../Backend/Items/spTypedefItem";
import { FormalParameter } from "./interfaces";
import { parsedLocToRange } from "./utils";
import { processDocStringComment } from "./processComment";

/**
 * Process an enum struct declaration.
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {TypedefDeclaration|FunctagDeclaration} res  Object containing the typedef/functag declaration details.
 * @returns void
 */
export function readTypedef(
  parserArgs: spParserArgs,
  res: TypedefDeclaration | FunctagDeclaration
): void {
  const range = parsedLocToRange(res.id.loc, parserArgs);
  const fullRange = parsedLocToRange(res.loc, parserArgs);
  const { doc, dep } = processDocStringComment(res.doc);
  let returnType = "";
  if (res.body.returnType) {
    returnType = res.body.returnType.id;
  }
  const typeDefItem = new TypedefItem(
    res.id.id,
    `typedef ${res.id.id} = function ${returnType} (${readTypeDefParams(
      res.body.params
    ).join(", ")});`,
    parserArgs.filePath,
    doc,
    returnType,
    range,
    fullRange,
    res.body.params
  );
  parserArgs.fileItems.items.push(typeDefItem);
  return;
}

/**
 * Extract variables from a TypeDef's body.
 * @param  {FormalParameter[]} params
 * @returns string
 */
export function readTypeDefParams(params: FormalParameter[]): string[] {
  if (params === null) {
    return [];
  }

  return params.map((e) => {
    // Handle "..." tokens.
    const id = e.id.id;
    let declType = "";
    if (e.declarationType) {
      if (Array.isArray(e.declarationType)) {
        declType = e.declarationType.join(" ");
      } else {
        declType = e.declarationType;
      }
      declType += " ";
    }

    return `${declType}${
      e.parameterType ? e.parameterType.name.id + " " : ""
    }${id}`;
  });
}
