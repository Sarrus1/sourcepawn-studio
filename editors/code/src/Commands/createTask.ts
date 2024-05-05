import { workspace as Workspace, window } from "vscode";
import {
  existsSync,
  writeFileSync,
  mkdirSync,
} from "fs";
import { join } from "path";
import { editConfig, getConfig, Section } from "../configUtils";

export function run(rootpath?: string): void {
  // Get configuration
  let includeDirs: string[] = getConfig(Section.LSP, "includeDirectories");
  let compilerPath: string = getConfig(Section.LSP, "compiler.path");
  if (!compilerPath) {
    window
      .showErrorMessage(
        "Sourcemod compiler not found in the project. You need to set 'compiler.path' for tasks generation to work.",
        "Open Settings"
      )
      .then((choice) => {
        if (choice === "Open Settings") {
          editConfig(Section.LSP, "compiler.path")
        }
      });
    return;
  }

  // Get workspace folder
  const workspaceFolders = Workspace.workspaceFolders;
  if (!workspaceFolders) {
    window.showErrorMessage("No workspaces are opened.");
    return;
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
    return;
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

    if (includeDirs.length > 0) {
      includeDirs.map(dir => json.tasks[0].args.push(`-i${dir}`));
    }
    writeFileSync(taskFilePath, JSON.stringify(json, null, 2), "utf8");
    window.showInformationMessage("Task file created successfully!")
  } catch (error) {
    window.showErrorMessage(`Could not create tasks.json file! ${error}`)
  }
}
