import { basename } from "path";

import { spParserArgs } from "./spParser";
import { EnumItem } from "../Backend/Items/spEnumItem";
import { EnumMemberItem } from "../Backend/Items/spEnumMemberItem";
import { ParserLocation } from "./interfaces";
import { parsedLocToRange } from "./utils";
import { Range } from "vscode";

interface ParsedEnumMember {
  id: string;
  loc: ParserLocation;
  doc: string | undefined;
}

export function readEnum(
  parserArgs: spParserArgs,
  id: ParsedEnumMember | undefined,
  loc: ParserLocation,
  body: ParsedEnumMember[]
) {
  try {
    let name: string;
    let nameRange: Range;
    if (!id) {
      parserArgs.anonEnumCount++;
      name = `Enum#${parserArgs.anonEnumCount}`;
      const newLoc = { ...loc };
      newLoc.start.column = 1;
      newLoc.end.column = 6;
      newLoc.end.line = newLoc.start.line;
      nameRange = parsedLocToRange(newLoc);
    } else {
      name = id.id;
      nameRange = parsedLocToRange(id.loc);
    }
    const key = name
      ? name
      : `${parserArgs.anonEnumCount}${basename(parserArgs.filePath)}`;
    const enumItem = new EnumItem(
      name,
      parserArgs.filePath,
      "",
      nameRange,
      parsedLocToRange(loc)
    );
    parserArgs.fileItems.set(key, enumItem);
    if (body) {
      body.forEach((e) => readEnumMember(parserArgs, e, enumItem));
    }
  } catch (e) {
    console.debug(e);
  }
}

function readEnumMember(
  parserArgs: spParserArgs,
  member: ParsedEnumMember,
  enumItem: EnumItem
) {
  const range = parsedLocToRange(member.loc);
  parserArgs.fileItems.set(
    member.id,
    new EnumMemberItem(
      member.id,
      parserArgs.filePath,
      member.doc,
      range,
      parserArgs.IsBuiltIn,
      enumItem
    )
  );
}
