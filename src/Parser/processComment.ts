import { DocString, ParsedComment } from "./interfaces";

/**
 * Processes a parsed comment and tries to extrapolate a doc comment from it.
 * This will handle `#pragma deprecated`.
 * @param  {(string|PreprocessorStatement)[]|string|undefined} docstring  The parsed comment to analyse.
 * @returns {DocString}
 */
export function processDocStringComment(docstring: ParsedComment): DocString {
  if (!docstring) {
    return { doc: undefined, dep: undefined };
  }
  if (Array.isArray(docstring)) {
    const txt: string[] = [];
    let dep: string;
    let emptyCount = 0;
    for (let e of docstring.reverse()) {
      if (emptyCount >= 2) {
        break;
      }
      if (e.type === "PragmaValue") {
        if (e.value.includes("deprecated")) {
          dep = e.value;
        }
      } else if (e.type === "SingleLineComment") {
        if (/^\s*$/.test(e.text)) {
          txt.push("\n\n");
          continue;
        }
        txt.push(e.text);
      } else if (
        e.type === "MultiLineComment" ||
        e.type === "MultiLineCommentNoLineTerminator"
      ) {
        txt.push(e.text);
      } else if (typeof e === "string" && /[\n\r]+/.test(e)) {
        emptyCount++;
      }
    }
    return { doc: txt.reverse().join("").trim(), dep };
  }

  return { doc: docstring.text, dep: undefined };
}
