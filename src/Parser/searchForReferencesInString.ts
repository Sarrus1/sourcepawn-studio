import { Parser } from "./spParser";
import { positiveRange } from "./utils";
import { URI } from "vscode-uri";
import { Location } from "vscode";

export function searchForReferencesInString(
  parser: Parser,
  line: string,
  offset = 0
): void {
  let isBlockComment = false;
  let isDoubleQuoteString = false;
  let isSingleQuoteString = false;
  let matchDefine: RegExpExecArray;
  const re = /(?:"|'|\/\/|\/\*|\*\/|\w+)/g;
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
      let item = parser.referencesMap.get(matchDefine[0]);

      if (item !== undefined) {
        const range = positiveRange(
          parser.lineNb,
          matchDefine.index + offset,
          matchDefine.index + matchDefine[0].length + offset
        );
        const location = new Location(URI.file(parser.file), range);
        item.references.push(location);
      }
    }
  } while (matchDefine);
  return;
}
