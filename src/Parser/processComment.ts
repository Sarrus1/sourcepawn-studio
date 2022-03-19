import { DocString } from "./interfaces";

/**
 * Processes a parsed comment and tries to extrapolate a doc comment from it.
 * This will handle `#pragma deprecated`.
 * @param  {string[]|undefined} txt  The parsed comment to analyse.
 * @returns {DocString}
 */
export function processDocStringComment(txt: string[] | undefined): DocString {
  if (txt === undefined || txt.length === 0) {
    return { doc: undefined, dep: undefined };
  }
  if (txt.length === 1) {
    return { doc: txt[0], dep: undefined };
  }
  if (txt.length === 2) {
    return { doc: txt.join("").trim(), dep: undefined };
  }
  const lastElt = txt[txt.length - 1];
  if (
    lastElt["type"] === "PragmaValue" &&
    lastElt["value"].startsWith("deprecated")
  ) {
    return {
      doc: txt[txt.length - 3],
      dep: lastElt["value"].replace("deprecated").trim(),
    };
  }
  return { doc: txt[txt.length - 2], dep: undefined };
}
