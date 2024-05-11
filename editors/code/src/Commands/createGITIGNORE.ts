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

  // Check if .gitignore already exists
  const gitignore = join(rootpath, ".gitignore");
  if (existsSync(gitignore)) {
    window.showErrorMessage(".gitignore file already exists.");
    return 1;
  }

  const myExtDir: string = extensions.getExtension("Sarrus.sourcepawn-vscode")
    .extensionPath;
  const changelogTemplatePath: string = join(
    myExtDir,
    "templates/gitignore_template"
  );
  copyFileSync(changelogTemplatePath, gitignore);
  window.showInformationMessage(".gitignore created successfully!")
  return 0;
}
