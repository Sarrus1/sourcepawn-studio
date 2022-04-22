import { spParserArgs } from "./spParser";
import { PropertyItem } from "../Backend/Items/spPropertyItem";
import {
  MethodDeclaration,
  MethodmapNativeForwardDeclaration,
  ParsedComment,
  ParsedID,
  ParserLocation,
} from "./interfaces";
import { parsedLocToRange } from "./utils";
import { MethodMapItem } from "../Backend/Items/spMethodmapItem";
import { processDocStringComment } from "./processComment";
import { readFunctionAndMethod } from "./readFunctionAndMethod";

export function readProperty(
  parserArgs: spParserArgs,
  id: ParsedID,
  loc: ParserLocation,
  parent: MethodMapItem,
  docstring: ParsedComment,
  returnType: ParsedID,
  body: (MethodDeclaration | MethodmapNativeForwardDeclaration)[],
  txt: string
): void {
  const range = parsedLocToRange(id.loc, parserArgs);
  const fullRange = parsedLocToRange(loc, parserArgs);
  const { doc, dep } = processDocStringComment(docstring);
  txt = txt.trim();
  const propertyItem = new PropertyItem(
    parent,
    id.id,
    parserArgs.filePath,
    txt,
    doc,
    range,
    fullRange,
    returnType.id
  );
  parserArgs.fileItems.set(`${id.id}-${parent.name}`, propertyItem);
  body.forEach((e) => {
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
      propertyItem
    );
  });
}
