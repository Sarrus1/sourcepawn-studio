import { DocString, ParsedComment, PreprocessorStatement } from "./interfaces";

/**
 * Process a parsed comment and try to extrapolate a doc comment from it.
 * This will handle `#pragma deprecated`.
 * @param  {ParsedComment} docstring  The parsed comment to analyse.
 * @returns {DocString}
 */
export function processDocStringComment(docstring: ParsedComment): DocString {
  if (!docstring) {
    return { doc: undefined, dep: undefined };
  }

  if (!Array.isArray(docstring)) {
    return { doc: docstring.text, dep: undefined };
  }

  const txt: string[] = [];
  let dep: string;
  let emptyCount = 0;
  docstring = docstring.reverse();
  for (let e of docstring) {
    if (emptyCount >= 2) {
      break;
    }
    if (e.type === "LineTerminatorSequence") {
      const statement = e.content[
        e.content.length - 1
      ] as PreprocessorStatement | null;
      if (!statement || statement.type !== "PragmaValue") {
        emptyCount++;
        continue;
      }
      if (/^deprecated/.test(statement.value)) {
        dep = statement.value.replace(/^deprecated\s*/, "");
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
    }
  }
  return { doc: txt.reverse().join("").trim(), dep };
}
