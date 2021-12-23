import { Parser } from "./spParser";
import { positiveRange } from "./utils";
import { URI } from "vscode-uri";
import { Location } from "vscode";

export function searchForDefinesInString(parser: Parser, line: string): void {
  if (line === undefined) {
    return;
  }
  let isBlockComment = false;
  let isDoubleQuoteString = false;
  let isSingleQuoteString = false;
  let matchDefine: RegExpExecArray;
  const re: RegExp = /(?:"|'|\/\/|\/\*|\*\/|\w+)/g;
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
      defineFile =
        parser.definesMap.get(matchDefine[0]) ||
        parser.enumMemberMap.get(matchDefine[0]);

      if (defineFile !== undefined) {
        let range = positiveRange(
          parser.lineNb,
          matchDefine.index,
          matchDefine.index + matchDefine[0].length
        );
        let location = new Location(URI.file(parser.file), range);
        // Treat defines from the current file differently or they will get
        // overwritten at the end of the parsing.
        if (defineFile === parser.file) {
          let define = parser.completions.get(matchDefine[0]);
          if (define === undefined) {
            continue;
          }
          define.calls.push(location);
          parser.completions.set(matchDefine[0], define);
          continue;
        }
        defineFile = defineFile.startsWith("file://")
          ? defineFile
          : URI.file(defineFile).toString();
        let items = parser.itemsRepository.items.get(defineFile);
        if (items === undefined) {
          continue;
        }
        let define = items.get(matchDefine[0]);
        if (define === undefined) {
          continue;
        }
        define.calls.push(location);
        items.set(matchDefine[0], define);
      }
    }
  } while (matchDefine);
  return;
}
