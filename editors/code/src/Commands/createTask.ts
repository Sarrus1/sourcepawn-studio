import { workspace as Workspace, window, commands } from "vscode";
import {
  existsSync,
  writeFileSync,
  mkdirSync,
} from "fs";
import { join } from "path";
import { getConfig, Section } from "../configUtils";

export function run(rootpath?: string) {
  // Get configuration
  let includeDirs: string[] = getConfig(Section.LSP, "includeDirectories");
  if (!includeDirs) {
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

  let compilerPath: string = getConfig(Section.LSP, "compiler.path");
  if (!compilerPath) {
    window
      .showErrorMessage(
        "Sourcemod compiler not found in the project. You need to set 'compiler.path' for tasks generation to work.",
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

  // Check if tasks file already exists
  const taskFilePath = join(rootpath, ".vscode/tasks.json");
  if (existsSync(taskFilePath)) {
    window.showErrorMessage("tasks.json file already exists.");
    return 3;
  }

  try {
    let json = {
      "version": "2.0.0",
      "tasks": [
        {
          "label": "Compile plugin",
          "type": "shell",
          "presentation": {
            "panel": "new"
          },
          "osx": {
            "command": compilerPath
          },
          "linux": {
            "command": compilerPath
          },
          "windows": {
            "command": compilerPath
          },
          "args": [
            "${file}",
            "-E",
            "-O2",
            "-v2",
            "-i${workspaceFolder}/scripting/include",
            "-o${workspaceFolder}/plugins/${fileBasenameNoExtension}.smx"
          ],
          "problemMatcher": {
            "owner": "sp",
            "fileLocation": "absolute",
            "pattern": {
              "regexp": "^(.*)\\((.+)\\)\\s:\\s(((warning|error|fatal error)\\s\\d+):\\s.*)$",
              "file": 1,
              "line": 2,
              "severity": 5,
              "message": 3
            }
          },
          "group": {
            "kind": "build",
            "isDefault": true
          }
        }
      ]
    }

    includeDirs.map(dir => json.tasks[0].args.push(`-i${dir}`));
    writeFileSync(taskFilePath, JSON.stringify(json, null, 2), "utf8");
    window.showInformationMessage("Task file created successfully!")
    return 0;
  } catch (error) {
    window.showErrorMessage(`Could not create tasks.json file! ${error}`)
    return 5;
  }
}
