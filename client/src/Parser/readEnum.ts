import { Parser } from "./spParser";
import { EnumStructItem, EnumItem, EnumMemberItem } from "../Providers/spItems";
import { State } from "./stateEnum";
import { searchForDefinesInString } from "./searchForDefinesInString";
import { basename } from "path";

export function readEnum(
  parser: Parser,
  match: RegExpMatchArray,
  line: string,
  IsStruct: boolean
) {
  let { description, params } = parser.parse_doc_comment();
  if (IsStruct) {
    // Create a completion for the enum struct itself if it has a name
    let enumStructName = match[1];
    let range = parser.makeDefinitionRange(enumStructName, line);
    var enumStructCompletion: EnumStructItem = new EnumStructItem(
      enumStructName,
      parser.file,
      description,
      range
    );
    parser.completions.add(enumStructName, enumStructCompletion);
    parser.state.push(State.EnumStruct);
    parser.state_data = {
      name: enumStructName,
    };
    return;
  }

  if (!match[1]) {
    parser.anonymousEnumCount++;
  }
  let nameMatch = match[1] ? match[1] : `Enum #${parser.anonymousEnumCount}`;
  let range = parser.makeDefinitionRange(match[1] ? match[1] : "enum", line);
  var enumCompletion: EnumItem = new EnumItem(
    nameMatch,
    parser.file,
    description,
    range
  );
  let key = match[1]
    ? match[1]
    : `${parser.anonymousEnumCount}${basename(parser.file)}`;
  parser.completions.add(key, enumCompletion);

  // Set max number of iterations for safety
  let iter = 0;
  // Match all the enum members
  while (iter < 100 && !/^\s*\}/.test(line)) {
    iter++;
    line = parser.lines.shift();
    parser.lineNb++;
    // Stop early if it's the end of the file
    if (line === undefined) {
      return;
    }
    let iterMatch = line.match(/^\s*(\w*)\s*.*/);

    // Skip if didn't match
    if (!iterMatch) {
      continue;
    }
    let enumMemberName = iterMatch[1];
    // Try to match multiblock comments
    let enumMemberDescription: string;
    iterMatch = line.match(/\/\*\*<?\s*(.+?(?=\*\/))/);
    if (iterMatch) {
      enumMemberDescription = iterMatch[1];
    }
    iterMatch = line.match(/\/\/<?\s*(.*)/);
    if (iterMatch) {
      enumMemberDescription = iterMatch[1];
    }
    let range = parser.makeDefinitionRange(enumMemberName, line);
    parser.completions.add(
      enumMemberName,
      new EnumMemberItem(
        enumMemberName,
        parser.file,
        enumMemberDescription,
        enumCompletion,
        range,
        parser.IsBuiltIn
      )
    );
    searchForDefinesInString(parser, line);
  }
  parser.addFullRange(key);
  return;
}
