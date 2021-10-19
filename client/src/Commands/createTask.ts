import { workspace as Workspace, window, commands, extensions } from "vscode";
import {
  existsSync,
  readFileSync,
  copyFileSync,
  writeFileSync,
  mkdirSync,
} from "fs";
import { join } from "path";

export function run(rootpath: string = undefined) {
  // Get configuration
  let sm_home: string = Workspace.getConfiguration("sourcepawn").get(
    "SourcemodHome"
  );
  if (!sm_home) {
    window
      .showWarningMessage(
        "SourceMod API not found in the project. You should set SourceMod Home for tasks generation to work. Do you want to install it automatically?",
        "Yes",
        "No, open Settings"
      )
      .then((choice) => {
        if (choice == "Yes") {
          commands.executeCommand("sourcepawn-vscode.installSM");
        } else if (choice === "No, open Settings") {
          commands.executeCommand(
            "workbench.action.openSettings",
            "@ext:sarrus.sourcepawn-vscode"
          );
        }
      });
  }

  let SpcompPath: string = Workspace.getConfiguration("sourcepawn").get(
    "SpcompPath"
  );
  if (!SpcompPath) {
    window
      .showErrorMessage(
        "SourceMod compiler not found in the project. You need to set SpcompPath for tasks generation to work.",
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
    return 1;
  }

  // get workspace folder
  let workspaceFolders = Workspace.workspaceFolders;
  if (!workspaceFolders) {
    window.showErrorMessage("No workspace are opened.");
    return 2;
  }

  //Select the rootpath
  if (rootpath === undefined) {
    rootpath = workspaceFolders?.[0].uri.fsPath;
  }

  // create task folder if it doesn't exist
  let taskFolderPath = join(rootpath, ".vscode");
  if (!existsSync(taskFolderPath)) {
    mkdirSync(taskFolderPath);
  }

  // Check if file already exists
  let taskFilePath = join(rootpath, ".vscode/tasks.json");
  if (existsSync(taskFilePath)) {
    window.showErrorMessage("tasks.json file already exists.");
    return 3;
  }
  let myExtDir: string = extensions.getExtension("Sarrus.sourcepawn-vscode")
    .extensionPath;
  let tasksTemplatesPath: string = join(myExtDir, "templates/tasks.json");
  copyFileSync(tasksTemplatesPath, taskFilePath);
  SpcompPath = SpcompPath.replace(/\\/gm, "\\\\");
  sm_home = sm_home.replace(/\\/gm, "\\\\");
  // Replace placeholders
  try {
    let data = readFileSync(taskFilePath, "utf8");
    let result = data.replace(/\${SpcompPath}/gm, SpcompPath);
    result = result.replace(/\${include_path}/gm, sm_home);
    writeFileSync(taskFilePath, result, "utf8");
  } catch (err) {
    console.log(err);
    return 4;
  }
  return 0;
}
