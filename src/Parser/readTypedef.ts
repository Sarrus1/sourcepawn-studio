import { spParserArgs } from "./interfaces";
import { TypedefItem } from "../Backend/Items/spTypedefItem";
import {
  FormalParameter,
  ParsedID,
  ParserLocation,
  TypedefBody,
  ParsedComment,
} from "./interfaces";
import { parsedLocToRange } from "./utils";
import { processDocStringComment } from "./processComment";

/**
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {ParsedID} id  The id of the Typedef.
 * @param  {ParserLocation} loc  The location of the Typedef.
 * @param  {TypedefBody} body  The body of the Typedef.
 * @param  {ParsedComment} docstring  The documentation of the Typedef.
 * @returns void
 */
export function readTypedef(
  parserArgs: spParserArgs,
  id: ParsedID,
  loc: ParserLocation,
  body: TypedefBody,
  docstring: ParsedComment
): void {
  const range = parsedLocToRange(id.loc, parserArgs);
  const fullRange = parsedLocToRange(loc, parserArgs);
  const { doc, dep } = processDocStringComment(docstring);
  let returnType = "";
  if (body.returnType) {
    returnType = body.returnType.id;
  }
  const typeDefItem = new TypedefItem(
    id.id,
    `typedef ${id.id} = function ${returnType} (${readTypeDefParams(
      body.params
    ).join(", ")});`,
    parserArgs.filePath,
    doc,
    returnType,
    range,
    fullRange,
    body.params
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
