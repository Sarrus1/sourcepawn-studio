import { spParserArgs } from "./spParser";
import { PropertyItem } from "../Backend/Items/spPropertyItem";
import { ParsedID, ParserLocation, PreprocessorStatement } from "./interfaces";
import { parsedLocToRange } from "./utils";
import { MethodMapItem } from "../Backend/Items/spMethodmapItem";
import { EnumStructItem } from "../Backend/Items/spEnumStructItem";
import { processDocStringComment } from "./processComment";

export function readProperty(
  parserArgs: spParserArgs,
  id: ParsedID,
  loc: ParserLocation,
  parent: MethodMapItem | EnumStructItem,
  docstring: (string | PreprocessorStatement)[] | undefined,
  returnType: ParsedID
): void {
  const range = parsedLocToRange(id.loc, parserArgs);
  const { doc, dep } = processDocStringComment(docstring);
  const propertyItem = new PropertyItem(
    parent,
    id.id,
    parserArgs.filePath,
    id.id,
    doc,
    range,
    returnType.id
  );
  parserArgs.fileItems.set(id.id + parent.name, propertyItem);
}
