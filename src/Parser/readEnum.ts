import { basename } from "path";

import { Parser } from "./spParser";
import { EnumStructItem } from "../Backend/Items/spEnumStructItem";
import { EnumItem } from "../Backend/Items/spEnumItem";
import { EnumMemberItem } from "../Backend/Items/spEnumMemberItem";
import { State } from "./stateEnum";
import { searchForDefinesInString } from "./searchForDefinesInString";
import { parseDocComment } from "./parseDocComment";
import { addFullRange } from "./addFullRange";

export function readEnum(
  parser: Parser,
  match: RegExpMatchArray,
  line: string,
  IsStruct: boolean
) {
  let { description, params } = parseDocComment(parser);
  if (IsStruct) {
    parseEnumStruct(parser, match[1], description, line);
    return;
  }

  if (!match[1]) {
    parser.anonymousEnumCount++;
  }
  let nameMatch = match[1] ? match[1] : `Enum#${parser.anonymousEnumCount}`;
  let range = parser.makeDefinitionRange(match[1] ? match[1] : "enum", line);
  var enumCompletion: EnumItem = new EnumItem(
    nameMatch,
    parser.file,
    description,
    range
  );
  const key = match[1]
    ? match[1]
    : `${parser.anonymousEnumCount}${basename(parser.file)}`;
  parser.completions.set(key, enumCompletion);

  // Set max number of iterations for safety
  let iter = 0;
  // Match all the enum members
  let foundEndToken = false;
  let i = match[0].length;
  let isBlockComment = false;
  let enumMemberName = "";
  description = "";

  while (!foundEndToken && iter < 10000) {
    iter++;
    if (line.length <= i) {
      line = parser.lines.shift();
      parser.lineNb++;
      if (line === undefined) {
        return;
      }
      searchForDefinesInString(parser, line);
      i = 0;
      continue;
    }

    if (isBlockComment) {
      let endComMatch = line.slice(i).match(/(.*)\*\//);
      if (endComMatch) {
        description += line.slice(i, i + endComMatch[1].length).trimEnd();
        isBlockComment = false;
        i += endComMatch[0].length;
        searchForDefinesInString(
          parser,
          line.slice(i + endComMatch[1].length + 1),
          endComMatch[1].length
        );
        let prevEnumMember: EnumMemberItem = parser.completions.get(
          enumMemberName
        );
        if (prevEnumMember !== undefined) {
          prevEnumMember.description = description;
        }
        enumMemberName = "";
        continue;
      }
      description += line.slice(i).trimEnd();
      line = parser.lines.shift();
      parser.lineNb++;
      if (line === undefined) {
        return;
      }
      i = 0;
      continue;
    }

    if (!isBlockComment) {
      if (line.length > i + 1) {
        if (line[i] == "/" && line[i + 1] == "*") {
          isBlockComment = true;
          i += 2;
          description = "";
          continue;
        }
        if (line[i] == "/" && line[i + 1] == "/") {
          let prevEnumMember: EnumMemberItem = parser.completions.get(
            enumMemberName
          );
          if (prevEnumMember !== undefined) {
            prevEnumMember.description = line.slice(i + 2).trim();
          }
          line = parser.lines.shift();
          parser.lineNb++;
          if (line === undefined) {
            return;
          }
          searchForDefinesInString(parser, line);
          i = 0;
          continue;
        }
      }
      if (line[i] == "}") {
        foundEndToken = true;
        continue;
      }
    }
    const croppedLine = line.slice(i);
    let iterMatch = croppedLine.match(
      /^\s*(?:\w+\s*:\s*)?([A-Za-z_]+\w*)(?:\s*\=.+?(?=(?:\r|\n|\,|\/\*|\/\/)))?/
    );
    if (!iterMatch || isBlockComment) {
      i++;
      continue;
    }
    enumMemberName = iterMatch[1];
    let range = parser.makeDefinitionRange(enumMemberName, line);
    parser.completions.set(
      enumMemberName,
      new EnumMemberItem(
        enumMemberName,
        parser.file,
        "",
        enumCompletion,
        range,
        parser.IsBuiltIn
      )
    );
    searchForDefinesInString(parser, line);
    i = iterMatch[0].length;
  }

  addFullRange(parser, key);
  return;
}

function parseEnumStruct(
  parser: Parser,
  enumStructName: string,
  desc: string,
  line: string
): void {
  let range = parser.makeDefinitionRange(enumStructName, line);
  var enumStructCompletion: EnumStructItem = new EnumStructItem(
    enumStructName,
    parser.file,
    desc,
    range
  );
  parser.completions.set(enumStructName, enumStructCompletion);
  parser.state.push(State.EnumStruct);
  parser.state_data = {
    name: enumStructName,
  };
}
