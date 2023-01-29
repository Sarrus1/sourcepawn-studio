import {
  Location,
  MarkdownString,
  Range,
  workspace as Workspace,
} from "vscode";
import { URI } from "vscode-uri";
import { existsSync, lstatSync } from "fs";
import { resolve, extname } from "path";

/**
 * Parse a Sourcemod JSDoc documentation string and convert it to a MarkdownString.
 * @param  {string} description   The Sourcemod JSDoc string.
 * @returns MarkdownString
 */
export function descriptionToMD(description?: string): MarkdownString {
  if (description === undefined || description === null) {
    return new MarkdownString("");
  }
  description = description
    // Remove leading *< from documentation (usually present in enum member's description)
    .replace(/^\*\</, "")
    // Remove leading * for block comments but keep indentation.
    .replace(/\*\s*\r?\n\s*\*/gm, "\n")
    // Remove leading * for block comments.
    .replace(/\r?\n\s*\*/gm, "")
    .replace(/^\*/, "")
    .replace(/\</gm, "\\<")
    .replace(/\>/gm, "\\>");
  // Make all @ nicer
  description = description.replace(/\s*(@[A-Za-z]+)\s+/gm, "\n\n_$1_ ");
  // Make the @param nicer
  description = description.replace(
    /(\_@param\_) ([A-Za-z0-9_.]+)\s*/gm,
    "$1 `$2` — "
  );

  // Format other functions which are referenced in the description
  description = description.replace(/(\w+\([A-Za-z0-9_ \:]*\))/gm, "`$1`");
  description = description.replace("DEPRECATED", "\n\n**DEPRECATED**");

  // Trim everything.
  description = description.trim();
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
    Workspace.getConfiguration("SourcePawnLanguageServer", workspaceFolder).get(
      "MainPath"
    ) || "";
  if (mainPath === "") {
    return undefined;
  }
  // Check if it exists, meaning it's an absolute path.
  if (!existsSync(mainPath) && workspaceFolders !== undefined) {
    // If it doesn't, loop over the workspace folders until one matches.
    for (const wk of workspaceFolders) {
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

/**
 * Returns a new location based on a filePath and a range.
 * @param  {string} filePath  The file's path of the new location.
 * @param  {Range} range      The range of the new location.
 * @returns Location
 */
export function locationFromRange(filePath: string, range: Range): Location {
  return new Location(URI.file(filePath), range);
}
