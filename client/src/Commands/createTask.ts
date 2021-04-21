import vscode = require("vscode");
import * as fs from "fs";
import * as path from "path";

export function run(args: any) {
  // Get configuration
  let sm_home: string = vscode.workspace
    .getConfiguration("sourcepawn")
    .get("SourcemodHome");
  if (!sm_home) {
    vscode.window
      .showWarningMessage(
        "SourceMod API not found in the project. You should set SourceMod Home for tasks generation to work.",
        "Open Settings"
      )
      .then((choice) => {
        if (choice === "Open Settings") {
          vscode.commands.executeCommand(
            "workbench.action.openWorkspaceSettings"
          );
        }
      });
  }

  let SpcompPath: string = vscode.workspace
    .getConfiguration("sourcepawn")
    .get("SpcompPath");
  if (!SpcompPath) {
    vscode.window
      .showErrorMessage(
        "SourceMod compiler not found in the project. You need to set SpcompPath for tasks generation to work.",
        "Open Settings"
      )
      .then((choice) => {
        if (choice === "Open Settings") {
          vscode.commands.executeCommand(
            "workbench.action.openWorkspaceSettings"
          );
        }
      });
    return 1;
  }

  // get workspace folder
  let workspaceFolders = vscode.workspace.workspaceFolders;
  if (!workspaceFolders) {
    vscode.window.showErrorMessage("No workspace are opened.");
    return 2;
  }

  //Select the rootpath
  let rootpath = workspaceFolders?.[0].uri;

  // create task folder if it doesn't exist
  let taskFolderPath = path.join(rootpath.fsPath, ".vscode");
  if (!fs.existsSync(taskFolderPath)) {
    fs.mkdirSync(taskFolderPath);
  }

  // Check if file already exists
  let taskFilePath = path.join(rootpath.fsPath, ".vscode/tasks.json");
  if (fs.existsSync(taskFilePath)) {
    vscode.window.showErrorMessage("tasks.json file already exists.");
    return 3;
  }
  let myExtDir: string = vscode.extensions.getExtension(
    "Sarrus.sourcepawn-vscode"
  ).extensionPath;
  let tasksTemplatesPath: string = path.join(myExtDir, "templates/tasks.json");
  fs.copyFileSync(tasksTemplatesPath, taskFilePath);
  SpcompPath = SpcompPath.replace(/\\/gm, "\\\\");
  sm_home = sm_home.replace(/\\/gm, "\\\\");
  // Replace placeholders
  try {
    let data = fs.readFileSync(taskFilePath, "utf8");
    let result = data.replace(/\${SpcompPath}/gm, SpcompPath);
    result = result.replace(/\${include_path}/gm, sm_home);
    fs.writeFileSync(taskFilePath, result, "utf8");
  } catch (err) {
    console.log(err);
    return 4;
  }
  return 0;
}
