import { workspace as Workspace, window, commands, extensions } from "vscode";
import { existsSync, readFileSync, copyFileSync, writeFileSync } from "fs";
import { join, basename } from "path";

export function run(rootpath?: string) {
  const GithubName: string = Workspace.getConfiguration("sourcepawn").get(
    "GithubName"
  );
  if (!GithubName) {
    window
      .showWarningMessage(
        "You didn't specify a GitHub username.",
        "Open Settings"
      )
      .then((choice) => {
        if (choice === "Open Settings") {
          commands.executeCommand(
            "workbench.action.openSettings",
            "@ext:sarrus.sourcepawn-vscode"
          );
        }
      });
  }

  // get workspace folder
  const workspaceFolders = Workspace.workspaceFolders;
  if (!workspaceFolders) {
    window.showErrorMessage("No workspace are opened.");
    return 1;
  }

  //Select the rootpath
  if (rootpath === undefined) {
    rootpath = workspaceFolders?.[0].uri.fsPath;
  }

  const rootname = basename(rootpath);

  // Check if README.md already exists
  const readmeFilePath = join(rootpath, "README.md");
  if (existsSync(readmeFilePath)) {
    window.showErrorMessage("README.md already exists, aborting.");
    return 2;
  }
  const myExtDir: string = extensions.getExtension("Sarrus.sourcepawn-vscode")
    .extensionPath;
  const tasksTemplatesPath: string = join(
    myExtDir,
    "templates/README_template.MD"
  );
  copyFileSync(tasksTemplatesPath, readmeFilePath);

  // Replace placeholders
  try {
    let result = readFileSync(readmeFilePath, "utf8");
    result = result.replace(/\${plugin_name}/gm, rootname);
    result = result.replace(/\${GithubName}/gm, GithubName);
    writeFileSync(readmeFilePath, result, "utf8");
  } catch (err) {
    console.error(err);
    return 3;
  }
  return 0;
}
