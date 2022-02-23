export type referencesSearchCallback = (match: RegExpExecArray) => void;

/**
 * Given a line of text, find references to items and save those references,
 * and ignore words in strings, and comments.
 *
 * The callbackfn handles what to do if a word if found. It handles the search for the corresponding variable.
 * @param  {string} line  The line to analyse.
 * @param  {referencesSearchCallback} callbackfn  The callback function which handles the search.
 * @returns void
 */
export function searchForReferencesInString(
  line: string,
  callbackfn: referencesSearchCallback
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
      callbackfn.call(this, matchDefine);
    }
  } while (matchDefine);
  return;
}
