import { spParserArgs } from "./spParser";
import { TypeDefItem } from "../Backend/Items/spTypedefItem";
import {
  ParsedParam,
  ParsedID,
  ParserLocation,
  TypeDefBody,
  ParsedComment,
} from "./interfaces";
import { parsedLocToRange } from "./utils";
import { processDocStringComment } from "./processComment";

/**
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {ParsedID} id  The id of the TypeDef.
 * @param  {ParserLocation} loc  The location of the TypeDef.
 * @param  {TypeDefBody} body  The body of the TypeDef.
 * @param  {ParsedComment} docstring  The documentation of the TypeDef.
 * @returns void
 */
export function readTypeDef(
  parserArgs: spParserArgs,
  id: ParsedID,
  loc: ParserLocation,
  body: TypeDefBody,
  docstring: ParsedComment
): void {
  const range = parsedLocToRange(id.loc, parserArgs);
  const fullRange = parsedLocToRange(loc, parserArgs);
  const { doc, dep } = processDocStringComment(docstring);
  let type = "void";
  if (body.returnType) {
    type = body.returnType.id;
  }
  const typeDefItem = new TypeDefItem(
    id.id,
    `typedef ${id.id} = function ${type} (${readTypeDefParams(body.params).join(
      ", "
    )});`,
    parserArgs.filePath,
    doc,
    type,
    range,
    fullRange
  );
  parserArgs.fileItems.items.push(typeDefItem);
  return;
}

/**
 * Extract variables from a TypeDef's body.
 * @param  {(ParsedParam[]|null)[]|null} params
 * @returns string
 */
function readTypeDefParams(
  params: (ParsedParam[] | null)[] | null
): string[] | undefined {
  if (!params) {
    return undefined;
  }
  if (params.length === 0) {
    return undefined;
  }
  return params[0].map((e) => {
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
