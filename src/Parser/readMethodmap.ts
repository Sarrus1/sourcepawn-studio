import { MethodmapDeclaration, spParserArgs } from "./interfaces";
import { MethodMapItem } from "../Backend/Items/spMethodmapItem";
import { globalIdentifier } from "../Misc/spConstants";
import { ParsedID } from "./interfaces";
import { parsedLocToRange } from "./utils";
import { processDocStringComment } from "./processComment";
import { readFunctionAndMethod } from "./readFunctionAndMethod";
import { readProperty } from "./readProperty";

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
  res.body.forEach((e) => {
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
        e.txt,
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
        e.txt,
        methodmapItem
      );
    } else {
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
