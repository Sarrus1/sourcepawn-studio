import { DocString, PreprocessorStatement } from "./interfaces";

/**
 * Processes a parsed comment and tries to extrapolate a doc comment from it.
 * This will handle `#pragma deprecated`.
 * @param  {(string|PreprocessorStatement)[]|string|undefined} docstring  The parsed comment to analyse.
 * @returns {DocString}
 */
export function processDocStringComment(
  docstring: (string | PreprocessorStatement)[] | string | undefined
): DocString {
  if (!docstring) {
    return { doc: undefined, dep: undefined };
  }
  if (Array.isArray(docstring)) {
    const txt: string[] = [];
    let dep: string;
    docstring.forEach((e) => {
      if (e["type"] !== undefined) {
        e = e as PreprocessorStatement;
        if (e["type"] === "PragmaValue") {
          dep = e.value;
        }
      } else {
        e = e as string;
        txt.push(e);
      }
    });
    return { doc: txt.join("").trim(), dep };
  }
  return { doc: docstring, dep: undefined };
}
