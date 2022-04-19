import { basename } from "path";
import { Range } from "vscode";

import { spParserArgs } from "./spParser";
import { EnumItem } from "../Backend/Items/spEnumItem";
import { EnumMemberItem } from "../Backend/Items/spEnumMemberItem";
import {
  ParserLocation,
  ParsedEnumMember,
  ParsedID,
  ParsedComment,
} from "./interfaces";
import { parsedLocToRange } from "./utils";
import { processDocStringComment } from "./processComment";

/**
 * Callback for a parsed enum.
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {ParsedID|undefined} id  The id of the enum.
 * @param  {ParserLocation} loc The location of the enum.
 * @param  {ParsedEnumMember[]} body  The members of the enum.
 * @param  {ParsedComment} doc The doc comment above the enum.
 * @param  {ParsedComment} lastDoc The doc comment of the last member of the enum.
 * @returns void
 */
export function readEnum(
  parserArgs: spParserArgs,
  id: ParsedID | undefined,
  loc: ParserLocation,
  body: ParsedEnumMember[],
  docstring: ParsedComment,
  lastDocstring: ParsedComment
): void {
  const { name, nameRange } = getEnumNameAndRange(parserArgs, id, loc);
  const key = name
    ? name
    : `${parserArgs.anonEnumCount}-${basename(parserArgs.filePath)}`;
  const { doc, dep } = processDocStringComment(docstring);
  const enumItem = new EnumItem(
    name,
    parserArgs.filePath,
    doc.length === 0 ? undefined : doc,
    nameRange,
    parsedLocToRange(loc, parserArgs)
  );
  parserArgs.fileItems.set(key, enumItem);
  if (body) {
    body.forEach((e, i) =>
      readEnumMember(
        parserArgs,
        e,
        enumItem,
        i === body.length - 1 ? lastDocstring : undefined
      )
    );
  }
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
  id: ParsedID | undefined,
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
 * Callback for a parsed enum member.
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {ParsedEnumMember} member  The member to be parsed.
 * @param  {EnumItem} enumItem  The parent of the enum member.
 * @param  {string} doc The doc associated to the enum member.
 * @returns void
 */
function readEnumMember(
  parserArgs: spParserArgs,
  member: ParsedEnumMember,
  enumItem: EnumItem,
  docstring: ParsedComment
): void {
  const range = parsedLocToRange(member.loc, parserArgs);
  if (docstring !== undefined) {
    var { doc, dep } = processDocStringComment(docstring);
  } else {
    var { doc, dep } = processDocStringComment(member.doc);
  }
  const memberItem = new EnumMemberItem(
    member.id,
    parserArgs.filePath,
    doc,
    range,
    parserArgs.IsBuiltIn,
    enumItem
  );
  parserArgs.fileItems.set(member.id, memberItem);
}
