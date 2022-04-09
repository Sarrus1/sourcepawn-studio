import { spParserArgs } from "./spParser";
import { MethodMapItem } from "../Backend/Items/spMethodmapItem";
import { globalItem } from "../Misc/spConstants";
import { ConstantItem } from "../Backend/Items/spConstantItem";
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

export function readMethodmap(
  parserArgs: spParserArgs,
  id: ParsedID | undefined,
  loc: ParserLocation,
  inherit: string | undefined,
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
  let parent: MethodMapItem | ConstantItem = globalItem;
  if (inherit !== undefined) {
    // TODO: Add inherit parsing
    parent = globalItem;
  }
  const methodmapItem = new MethodMapItem(
    id.id,
    parent,
    `methodmap ${id.id}${inherit ? " < " + inherit : ""}`,
    doc,
    parserArgs.filePath,
    range,
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
    }
  });
}
