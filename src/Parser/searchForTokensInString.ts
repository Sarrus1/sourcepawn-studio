import { Parser } from "./spParser";
import { positiveRange } from "./utils";
import { URI } from "vscode-uri";
import { Location } from "vscode";
import { DefineItem } from "../Backend/Items/spDefineItem";
import { EnumMemberItem } from "../Backend/Items/spEnumMemberItem";
import { constants } from "os";
import { FunctionItem } from "../Backend/Items/spFunctionItem";

export function searchForTokensInString(
  parser: Parser,
  line: string,
  offset = 0
): void {
  if (line === undefined) {
    return;
  }
  let isBlockComment = false;
  let isDoubleQuoteString = false;
  let isSingleQuoteString = false;
  let matchDefine: RegExpExecArray;
  const re: RegExp = /(?:"|'|\/\/|\/\*|\*\/|\w+)/g;
  let item: DefineItem | EnumMemberItem | FunctionItem;
  let defineFile: string;
  do {
    matchDefine = re.exec(line);
    if (matchDefine) {
      if (matchDefine[0] === '"' && !isSingleQuoteString) {
        isDoubleQuoteString = !isDoubleQuoteString;
      } else if (matchDefine[0] === "'" && !isDoubleQuoteString) {
        isSingleQuoteString = !isSingleQuoteString;
      } else if (
        matchDefine[0] === "//" &&
        !isDoubleQuoteString &&
        !isSingleQuoteString
      ) {
        break;
      } else if (
        matchDefine[0] === "/*" ||
        (matchDefine[0] === "*/" &&
          !isDoubleQuoteString &&
          !isSingleQuoteString)
      ) {
        isBlockComment = !isBlockComment;
      }
      if (isBlockComment || isDoubleQuoteString || isSingleQuoteString) {
        continue;
      }
      if (["float", "bool", "char", "int"].includes(matchDefine[0])) {
        continue;
      }
      item =
        parser.tokensMap.definesMap.get(matchDefine[0]) ||
        parser.tokensMap.enumMembersMap.get(matchDefine[0]) ||
        parser.tokensMap.functionsMap.get(matchDefine[0]);

      if (item !== undefined) {
        defineFile = item.filePath;
        let range = positiveRange(
          parser.lineNb,
          matchDefine.index + offset,
          matchDefine.index + matchDefine[0].length + offset
        );
        let location = new Location(URI.file(parser.file), range);
        // Treat tokens from the current file differently or they will get
        // overwritten at the end of the parsing.
        if (defineFile === parser.file) {
          let localItem = parser.completions.get(matchDefine[0]);
          if (localItem === undefined) {
            continue;
          }
          localItem.calls.push(location);
          parser.completions.set(matchDefine[0], localItem);
          continue;
        }
        defineFile = defineFile.startsWith("file://")
          ? defineFile
          : URI.file(defineFile).toString();
        item.calls.push(location);
      }
    }
  } while (matchDefine);
  return;
}
