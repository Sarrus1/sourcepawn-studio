import { MarkdownString } from "vscode";

export function descriptionToMD(description: string): MarkdownString {
  if (typeof description === "undefined") return new MarkdownString("");
  description = description
    .replace(/\</gm, "\\<")
    .replace(/\>/gm, "\\>")
    .replace(/([^.])(\.) *[\n]+(?:\s*([^@\s.]))/gm, "$1. $3")
    .replace(/\s+\*\s*/gm, "\n\n");
  // Make all @ nicer
  description = description.replace(/\s*(@[A-Za-z]+)\s+/gm, "\n\n_$1_ ");
  // Make the @params nicer
  description = description.replace(
    /(\_@param\_) ([A-Za-z0-9_.]+)\s*/gm,
    "$1 `$2` — "
  );

  // Format other functions which are referenced in the description
  description = description.replace(
    /([A-Za-z0-9_]+\([A-Za-z0-9_ \:]*\))/gm,
    "`$1`"
  );
  return new MarkdownString(description);
}
