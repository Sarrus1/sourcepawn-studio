import { workspace as Workspace, window, extensions } from "vscode";
import { existsSync, copyFileSync } from "fs";
import { join } from "path";

export function run(rootpath: string = undefined) {
  // get workspace folder
  let workspaceFolders = Workspace.workspaceFolders;
  if (workspaceFolders === undefined) {
    window.showErrorMessage("No workspace are opened.");
    return 1;
  }

  //Select the rootpath
  if (rootpath === undefined || typeof rootpath !== "string") {
    rootpath = workspaceFolders?.[0].uri.fsPath;
  }

  // Check if CHANGELOG.md already exists
  let changelogFilePath = join(rootpath, "CHANGELOG.md");
  if (existsSync(changelogFilePath)) {
    window.showErrorMessage("CHANGELOG.md already exists, aborting.");
    return 2;
  }
  let myExtDir: string = extensions.getExtension("Sarrus.sourcepawn-vscode")
    .extensionPath;
  let changelogTemplatePath: string = join(
    myExtDir,
    "templates/CHANGELOG_template.md"
  );
  copyFileSync(changelogTemplatePath, changelogFilePath);
  return 0;
}
