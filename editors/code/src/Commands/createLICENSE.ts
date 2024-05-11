import { workspace as Workspace, window, extensions } from "vscode";
import { existsSync, copyFileSync } from "fs";
import { join } from "path";

export function run(rootpath?: string): number {
  // Get workspace folder
  const workspaceFolders = Workspace.workspaceFolders;
  if (!workspaceFolders) {
    window.showErrorMessage("No workspaces are opened.");
    return 1;
  }

  // Select the rootpath
  if (!rootpath || typeof rootpath !== "string") {
    rootpath = workspaceFolders?.[0].uri.fsPath;
  }

  // Check if CHANGELOG.md already exists
  const changelogFilePath = join(rootpath, "LICENSE");
  if (existsSync(changelogFilePath)) {
    window.showErrorMessage("LICENSE file already exists.");
    return 1;
  }
  const myExtDir: string = extensions.getExtension("Sarrus.sourcepawn-vscode")
    .extensionPath;
  const changelogTemplatePath: string = join(
    myExtDir,
    "templates/LICENSE_template"
  );
  copyFileSync(changelogTemplatePath, changelogFilePath);
  window.showInformationMessage("LICENSE created successfully!");
  return 0;
}
