import { workspace as Workspace, window, commands, extensions } from "vscode";
import {
  existsSync,
  readFileSync,
  copyFileSync,
  writeFileSync,
  mkdirSync,
} from "fs";
import { join } from "path";
import { getConfig, Section } from "../configUtils";

export function run(rootpath?: string) {
  // Get configuration
  let smHome: string = getConfig(Section.SourcePawn, "SourcemodHome");
  if (!smHome) {
    window
      .showWarningMessage(
        "Sourcemod API not found in the project. You should set Sourcemod Home for tasks generation to work. Do you want to install it automatically?",
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

  let spcompPath: string = getConfig(Section.LSP, "spcompPath");
  if (!spcompPath) {
    window
      .showErrorMessage(
        "Sourcemod compiler not found in the project. You need to set spcompPath for tasks generation to work.",
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

  // Get workspace folder
  const workspaceFolders = Workspace.workspaceFolders;
  if (!workspaceFolders) {
    window.showErrorMessage("No workspaces are opened.");
    return 2;
  }

  // Select the rootpath
  if (rootpath === undefined) {
    rootpath = workspaceFolders?.[0].uri.fsPath;
  }

  // Create task folder if it doesn't exist
  const taskFolderPath = join(rootpath, ".vscode");
  if (!existsSync(taskFolderPath)) {
    mkdirSync(taskFolderPath);
  }

  // Check if file already exists
  const taskFilePath = join(rootpath, ".vscode/tasks.json");
  if (existsSync(taskFilePath)) {
    window.showErrorMessage("tasks.json file already exists.");
    return 3;
  }
  const myExtDir: string = extensions.getExtension(
    "Sarrus.sourcepawn-vscode"
  ).extensionPath;
  const tasksTemplatesPath: string = join(myExtDir, "templates/tasks.json");
  copyFileSync(tasksTemplatesPath, taskFilePath);
  spcompPath = spcompPath.replace(/\\/gm, "\\\\");
  smHome = smHome.replace(/\\/gm, "\\\\");
  // Replace placeholders
  try {
    const data = readFileSync(taskFilePath, "utf8");
    let result = data.replace(/\${spcompPath}/gm, spcompPath);
    result = result.replace(/\${include_path}/gm, smHome);
    writeFileSync(taskFilePath, result, "utf8");
  } catch (err) {
    console.error(err);
    return 4;
  }
  return 0;
}
