import { spParserArgs } from "./spParser";
import { MethodMapItem } from "../Backend/Items/spMethodmapItem";
import { globalIdentifier } from "../Misc/spConstants";
import {
  MethodDeclaration,
  MethodmapNativeForwardDeclaration,
  ParsedID,
  ParserLocation,
  PreprocessorStatement,
  PropertyDeclaration,
} from "./interfaces";
import { parsedLocToRange } from "./utils";
import { processDocStringComment } from "./processComment";
import { readFunctionAndMethod } from "./readFunctionAndMethod";
import { readProperty } from "./readProperty";

export function readMethodmap(
  parserArgs: spParserArgs,
  id: ParsedID | undefined,
  loc: ParserLocation,
  inherit: ParsedID | "__nullable__" | undefined,
  docstring: (string | PreprocessorStatement)[] | undefined,
  body: {
    type: "MethodmapBody";
    body: (
      | PropertyDeclaration
      | MethodDeclaration
      | MethodmapNativeForwardDeclaration
    )[];
  }
): void {
  const range = parsedLocToRange(id.loc);
  const fullRange = parsedLocToRange(loc);
  const { doc, dep } = processDocStringComment(docstring);
  const methodmapItem = new MethodMapItem(
    id.id,
    !(inherit && inherit === "__nullable__")
      ? (inherit as ParsedID).id
      : globalIdentifier,
    `methodmap ${id.id}${inherit ? " < " + inherit : ""}`,
    doc,
    parserArgs.filePath,
    range,
    fullRange,
    parserArgs.IsBuiltIn
  );
  parserArgs.fileItems.set(id.id, methodmapItem);
  body["body"].forEach((e) => {
    if (e.type === "MethodDeclaration") {
      readFunctionAndMethod(
        parserArgs,
        e.accessModifier,
        e.returnType,
        e.id,
        e.loc,
        e.doc,
        e.params,
        e.body,
        methodmapItem
      );
    } else if (e.type === "MethodmapNativeForwardDeclaration") {
      readFunctionAndMethod(
        parserArgs,
        e.accessModifier,
        e.returnType,
        e.id,
        e.loc,
        e.doc,
        e.params,
        null,
        methodmapItem
      );
    } else {
      readProperty(
        parserArgs,
        e.id,
        e.loc,
        methodmapItem,
        e.doc,
        e.propertyType
      );
    }
  });
}
