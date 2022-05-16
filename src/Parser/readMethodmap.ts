import {
  MethodmapBody,
  MethodmapDeclaration,
  spParserArgs,
} from "./interfaces";
import { MethodMapItem } from "../Backend/Items/spMethodmapItem";
import { globalIdentifier } from "../Misc/spConstants";
import { ParsedID } from "./interfaces";
import { parsedLocToRange } from "./utils";
import { processDocStringComment } from "./processComment";
import { readFunctionAndMethod } from "./readFunctionAndMethod";
import { readProperty } from "./readProperty";

/**
 * Process a methodmap declaration.
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {MethodmapDeclaration} res  The object containing the methodmap declaration details.
 * @returns void
 */
export function readMethodmap(
  parserArgs: spParserArgs,
  res: MethodmapDeclaration
): void {
  const range = parsedLocToRange(res.id.loc, parserArgs);
  const fullRange = parsedLocToRange(res.loc, parserArgs);
  const { doc, dep } = processDocStringComment(res.doc);
  const methodmapItem = new MethodMapItem(
    res.id.id,
    res.inherit && res.inherit !== "__nullable__"
      ? (res.inherit as ParsedID).id
      : globalIdentifier,
    doc,
    parserArgs.filePath,
    range,
    fullRange,
    parserArgs.IsBuiltIn
  );
  parserArgs.fileItems.items.push(methodmapItem);
  parseMethodmapBody(parserArgs, res.body, methodmapItem);
}

/**
 * Process methodmap's body.
 * @param  {spParserArgs} parserArgs  ParserArgs objects passed to the parser.
 * @param  {MethodmapBody} body  Parsed body of the methodmap.
 * @param  {MethodMapItem} methodmapItem  Methodmap item associated to the body.
 * @returns void
 */
function parseMethodmapBody(
  parserArgs: spParserArgs,
  body: MethodmapBody,
  methodmapItem: MethodMapItem
): void {
  body.forEach((e) => {
    switch (e.type) {
      case "MethodDeclaration":
        readFunctionAndMethod(
          parserArgs,
          e.accessModifier,
          e.returnType,
          e.id,
          e.loc,
          e.doc,
          e.params,
          e.body,
          e.txt,
          methodmapItem
        );
        break;
      case "MethodmapNativeForwardDeclaration":
        readFunctionAndMethod(
          parserArgs,
          e.accessModifier,
          e.returnType,
          e.id,
          e.loc,
          e.doc,
          e.params,
          null,
          e.txt,
          methodmapItem
        );
        break;
      case "PropertyDeclaration":
        readProperty(
          parserArgs,
          e.id,
          e.loc,
          methodmapItem,
          e.doc,
          e.propertyType,
          e.body,
          e.txt
        );
    }
  });
}
