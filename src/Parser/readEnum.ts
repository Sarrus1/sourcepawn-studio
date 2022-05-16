import { Range } from "vscode";

import { EnumDeclaration, spParserArgs } from "./interfaces";
import { EnumItem } from "../Backend/Items/spEnumItem";
import { EnumMemberItem } from "../Backend/Items/spEnumMemberItem";
import { ParserLocation, EnumMemberDeclaration, ParsedID } from "./interfaces";
import { parsedLocToRange } from "./utils";
import { processDocStringComment } from "./processComment";

/**
 * Callback for a parsed enum.
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {EnumDeclaration} res  Object containing the enum declaration details.
 * @returns void
 */
export function readEnum(parserArgs: spParserArgs, res: EnumDeclaration): void {
  const { name, nameRange } = getEnumNameAndRange(parserArgs, res.id, res.loc);
  const { doc, dep } = processDocStringComment(res.doc);
  const enumItem = new EnumItem(
    name,
    parserArgs.filePath,
    doc,
    nameRange,
    parsedLocToRange(res.loc, parserArgs)
  );
  parserArgs.fileItems.items.push(enumItem);
  readEnumMembers(parserArgs, res.body, enumItem);
}

/**
 * Generate the name and the range of a potential anonymous enum.
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {ParsedID|undefined} id  The id of the enum.
 * @param  {ParserLocation} loc The location of the enum.
 * @returns Range
 */
function getEnumNameAndRange(
  parserArgs: spParserArgs,
  id: ParsedID | null,
  loc: ParserLocation
): { name: string; nameRange: Range } {
  let name: string;
  let nameRange: Range;
  if (!id) {
    parserArgs.anonEnumCount++;
    name = `Enum#${parserArgs.anonEnumCount}`;
    const newLoc = { ...loc };
    newLoc.start.column = 1;
    newLoc.end.column = 6;
    newLoc.end.line = newLoc.start.line;
    nameRange = parsedLocToRange(newLoc, parserArgs);
  } else {
    name = id.id;
    nameRange = parsedLocToRange(id.loc, parserArgs);
  }
  return { name, nameRange };
}

/**
 * Process the body of an enum.
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {EnumMemberDeclaration[]} body  The body of the enum to parse.
 * @param  {EnumItem} enumItem  The parent of the enum members.
 * @returns void
 */
function readEnumMembers(
  parserArgs: spParserArgs,
  body: EnumMemberDeclaration[],
  enumItem: EnumItem
): void {
  if (!body) {
    return;
  }
  body.forEach((e) => {
    const range = parsedLocToRange(e.id.loc, parserArgs);
    const { doc, dep } = processDocStringComment(e.doc);
    const memberItem = new EnumMemberItem(
      e.id.id,
      parserArgs.filePath,
      doc,
      range,
      parserArgs.IsBuiltIn,
      enumItem
    );
    parserArgs.fileItems.items.push(memberItem);
  });
}
