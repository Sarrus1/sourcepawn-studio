import { workspace as Workspace, window, extensions } from "vscode";
import { existsSync, copyFileSync } from "fs";
import { join } from "path";

export function run(rootpath?: string) {
  // get workspace folder
  const workspaceFolders = Workspace.workspaceFolders;
  if (workspaceFolders === undefined) {
    window.showErrorMessage("No workspace are opened.");
    return 1;
  }

  //Select the rootpath
  if (rootpath === undefined || typeof rootpath !== "string") {
    rootpath = workspaceFolders?.[0].uri.fsPath;
  }

  // Check if CHANGELOG.md already exists
  const changelogFilePath = join(rootpath, "CHANGELOG.md");
  if (existsSync(changelogFilePath)) {
    window.showErrorMessage("CHANGELOG.md already exists, aborting.");
    return 2;
  }
  const myExtDir: string = extensions.getExtension("Sarrus.sourcepawn-vscode")
    .extensionPath;
  const changelogTemplatePath: string = join(
    myExtDir,
    "templates/CHANGELOG_template.md"
  );
  copyFileSync(changelogTemplatePath, changelogFilePath);
  return 0;
}
