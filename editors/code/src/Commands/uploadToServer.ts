import { join } from "path";

import { run as runServerCommands } from "./runServerCommands";
import { alwaysCompileMainPath, getMainCompilationFile } from "../spUtils";
import { WorkspaceFolder, commands, window, workspace } from "vscode";
import { lastActiveEditor } from "../spIndex";
import { URI } from "vscode-uri";
const FTPDeploy = require("ftp-deploy");

export async function run(args?: string) {
  const ftpDeploy = new FTPDeploy();
  let workspaceFolder: WorkspaceFolder;
  let fileToUpload: string;

  // If we receive arguments, the file to upload has already been figured out for us,
  // else, we use the user's choice, main compilation file or current editor
  if (!args) {
    if (await alwaysCompileMainPath()) {
      fileToUpload = await getMainCompilationFile();
    }
    else {
      fileToUpload = lastActiveEditor.document.uri.fsPath
    }
  }
  else {
    fileToUpload = args;
    workspaceFolder = workspace.getWorkspaceFolder(URI.file(args));
  }

  // Return if upload settings are not defined
  const config: object = workspace
    .getConfiguration("sourcepawn", workspaceFolder)
    .get("UploadOptions");
  if (config === undefined) {
    window
      .showErrorMessage("Upload settings are empty.", "Open Settings")
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

  // Return if upload settings are not properly configured
  if (config["user"] == "" || config["host"] == "") {
    window
      .showErrorMessage(
        "Some settings are improperly defined in the upload settings.",
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
    return 2;
  }

  // Override the "deleteRemote" setting for safety.
  config["deleteRemote"] = false;

  // If specified, replace macro with main compilation file
  if (config["localRoot"] === "${mainPath}") {
    config["localRoot"] = getMainCompilationFile();
  }

  if (config["isRootRelative"]) {
    // Concat the workspace with it's root if the path is relative.
    if (workspaceFolder === undefined) {
      window.showWarningMessage(
        "No workspace or folder found, with isRootRelative is set to true.\nSet it to false, or open the file from a workspace."
      );
      return 1;
    }
    const workspaceRoot = workspaceFolder.uri.fsPath;
    config["localRoot"] = join(workspaceRoot, config["localRoot"]);
  }

  // Copy the config object to avoid https://github.com/microsoft/vscode/issues/80976
  const ftpConfig = { ...config };
  // Delete that setting to avoid problems with the ftp/sftp library
  delete ftpConfig["isRootRelative"];

  console.log("Starting the upload");
  console.log(ftpConfig);
  ftpDeploy
    .deploy(ftpConfig)
    .then(() => {
      console.log("Upload is finished.");
      if (
        workspace
          .getConfiguration("sourcepawn", workspaceFolder)
          .get<string>("runServerCommands") === "afterUpload"
      ) {
        runServerCommands(fileToUpload);
      }
    })
    .catch(err => {
      // TODO: inform the user
      return console.error(err);
    });
  return 0;
}
