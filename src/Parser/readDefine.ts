import { spParserArgs } from "./spParser";
import { DefineItem } from "../Backend/Items/spDefineItem";
import { ParsedDefine, ParserLocation } from "./interfaces";
import { parsedLocToRange } from "./utils";

export function readDefine(
  parserArgs: spParserArgs,
  id: ParsedDefine,
  loc: ParserLocation,
  value: string | null,
  doc: string
): void {
  const range = parsedLocToRange(id.loc);
  const fullRange = parsedLocToRange(loc);
  const defineItem = new DefineItem(
    id.id,
    value,
    doc,
    parserArgs.filePath,
    range,
    parserArgs.IsBuiltIn,
    fullRange
  );
  parserArgs.fileItems.set(id.id, defineItem);
  return;
}
