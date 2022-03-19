import { spParserArgs } from "./spParser";
import { TypeDefItem } from "../Backend/Items/spTypedefItem";
import {
  ParamsEntity,
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
  const range = parsedLocToRange(id.loc);
  const fullRange = parsedLocToRange(loc);
  const { doc, dep } = processDocStringComment(docstring);
  const typeDefItem = new TypeDefItem(
    id.id,
    `typedef ${id.id} = function ${body.returnType} (${readTypeDefBody(
      body.params
    ).join(", ")});`,
    parserArgs.filePath,
    doc,
    body.returnType.id,
    range,
    fullRange
  );
  parserArgs.fileItems.set(id.id, typeDefItem);
}

/**
 * Extract variables from a TypeDef's body.
 * @param  {(ParamsEntity[]|null)[]|null} body
 * @returns string
 */
function readTypeDefBody(
  body: (ParamsEntity[] | null)[] | null
): string[] | undefined {
  if (!body) {
    return undefined;
  }
  if (body.length === 0) {
    return undefined;
  }
  return body[0].map((e) => e.parameterType.id + " " + e.id.id);
}
