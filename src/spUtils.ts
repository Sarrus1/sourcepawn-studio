import { MarkdownString, workspace as Workspace } from "vscode";
import { URI } from "vscode-uri";
import { existsSync, lstatSync } from "fs";
import { resolve, extname } from "path";

/**
 * Parse a Sourcemod JSDoc documentation string and convert it to a MarkdownString.
 * @param  {string} description   The Sourcemod JSDoc string.
 * @returns MarkdownString
 */
export function descriptionToMD(description?: string): MarkdownString {
  if (description === undefined) {
    return new MarkdownString("");
  }
  description = description
    // Remove leading *< from documentation (usually present in enum member's description)
    .replace(/^\*\</, "")
    .replace(/\</gm, "\\<")
    .replace(/\>/gm, "\\>")
    .replace(/([\w\,]{1})\n/gm, "$1")
    //.replace(/([^.])(\.) *[\n]+(?:\s*([^@\s.]))/gm, "$1. $3")
    .replace(/\s+\*\s*/gm, "\n\n");
  // Make all @ nicer
  description = description.replace(/\s*(@[A-Za-z]+)\s+/gm, "\n\n_$1_ ");
  // Make the @param nicer
  description = description.replace(
    /(\_@param\_) ([A-Za-z0-9_.]+)\s*/gm,
    "$1 `$2` — "
  );

  // Format other functions which are referenced in the description
  description = description.replace(/(\w*.\w+\([A-Za-z0-9_ \:]*\))/gm, "`$1`");
  description = description.replace("DEPRECATED", "**DEPRECATED**");
  return new MarkdownString(description);
}

/**
 * Find the MainPath setting for a given URI.
 * Will return an empty string if the mainpath setting doesn't point to an
 * existing location, and will return undefined if nothing is found.
 * @param  {Uri} uri?   The URI we are looking up the MainPath for.
 * @returns string | undefined
 */
export function findMainPath(uri?: URI): string | undefined {
  const workspaceFolders = Workspace.workspaceFolders;
  const workspaceFolder =
    uri === undefined ? undefined : Workspace.getWorkspaceFolder(uri);
  let mainPath: string =
    Workspace.getConfiguration("sourcepawn", workspaceFolder).get("MainPath") ||
    "";
  if (mainPath === "") {
    return undefined;
  }
  // Check if it exists, meaning it's an absolute path.
  if (!existsSync(mainPath) && workspaceFolders !== undefined) {
    // If it doesn't, loop over the workspace folders until one matches.
    for (let wk of workspaceFolders) {
      mainPath = resolve(wk.uri.fsPath, mainPath);
      if (existsSync(mainPath)) {
        return mainPath;
      }
    }
    return "";
  } else {
    return mainPath;
  }
}

export function checkMainPath(mainPath: string): boolean {
  if (!existsSync(mainPath)) {
    return false;
  }
  if (!lstatSync(mainPath).isFile()) {
    return false;
  }
  return extname(mainPath) === ".sp";
}
