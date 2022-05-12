import { spParserArgs } from "./interfaces";
import { TypeDefItem } from "../Backend/Items/spTypedefItem";
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
 * @param  {ParsedID} id  The id of the TypeDef.
 * @param  {ParserLocation} loc  The location of the TypeDef.
 * @param  {TypedefBody} body  The body of the TypeDef.
 * @param  {ParsedComment} docstring  The documentation of the TypeDef.
 * @returns void
 */
export function readTypeDef(
  parserArgs: spParserArgs,
  id: ParsedID,
  loc: ParserLocation,
  body: TypedefBody,
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
 * @param  {(FormalParameter[]} params
 * @returns string
 */
function readTypeDefParams(params: FormalParameter[]): string[] | undefined {
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
