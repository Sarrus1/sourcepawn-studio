import { workspace as Workspace } from "vscode";
import { URI } from "vscode-uri";
import { existsSync } from "fs";
import { resolve } from "path";

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
      "mainPath"
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

/**
 * If needed, migrate the settings of the user to use the LanguageServer.
 */
export function migrateSettings() {
  const smHome: string =
    Workspace.getConfiguration("sourcepawn").get("SourcemodHome");
  const optionalIncludeDirsPaths: string[] = Workspace.getConfiguration(
    "sourcepawn"
  ).get("optionalIncludeDirsPaths");

  const includesDirectories: string[] = Workspace.getConfiguration(
    "SourcePawnLanguageServer"
  ).get("includesDirectories");

  const oldSpcompPath: string =
    Workspace.getConfiguration("sourcepawn").get("SpcompPath");

  const newSpcompPath: string = Workspace.getConfiguration(
    "SourcePawnLanguageServer"
  ).get("spcompPath");

  if (
    (includesDirectories.length == 0 && smHome) ||
    (!newSpcompPath && oldSpcompPath)
  ) {
    Workspace.getConfiguration("SourcePawnLanguageServer").update(
      "includesDirectories",
      Array.from(new Set([smHome].concat(optionalIncludeDirsPaths))),
      true
    );

    if (oldSpcompPath && !newSpcompPath) {
      Workspace.getConfiguration("SourcePawnLanguageServer").update(
        "spcompPath",
        oldSpcompPath,
        true
      );
    }
  }
}

export function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
