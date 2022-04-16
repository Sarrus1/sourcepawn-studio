import { spParserArgs } from "./spParser";
import { TypeDefItem } from "../Backend/Items/spTypedefItem";
import {
  ParsedParam,
  ParsedID,
  ParserLocation,
  TypeDefBody,
} from "./interfaces";
import { parsedLocToRange } from "./utils";
import { processDocStringComment } from "./processComment";

/**
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {ParsedID} id  The id of the TypeDef.
 * @param  {ParserLocation} loc  The location of the TypeDef.
 * @param  {TypeDefBody} body  The body of the TypeDef.
 * @param  {string[]|undefined} docstring  The documentation of the TypeDef.
 * @returns void
 */
export function readTypeDef(
  parserArgs: spParserArgs,
  id: ParsedID,
  loc: ParserLocation,
  body: TypeDefBody,
  docstring: string[] | undefined
): void {
  const range = parsedLocToRange(id.loc, parserArgs);
  const fullRange = parsedLocToRange(loc, parserArgs);
  const { doc, dep } = processDocStringComment(docstring);
  const typeDefItem = new TypeDefItem(
    id.id,
    `typedef ${id.id} = function ${body.returnType.id} (${readTypeDefParams(
      body.params
    ).join(", ")});`,
    parserArgs.filePath,
    doc,
    body.returnType.id,
    range,
    fullRange
  );
  parserArgs.fileItems.set(id.id, typeDefItem);
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
