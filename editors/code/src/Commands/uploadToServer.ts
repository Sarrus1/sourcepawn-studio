import { join } from "path";

import { run as runServerCommands } from "./runServerCommands";
import { getMainCompilationFile } from "../spUtils";
import { WorkspaceFolder, commands, window, workspace } from "vscode";
import { lastActiveEditor } from "../spIndex";
import { URI } from "vscode-uri";
import { Section, getConfig } from "../configUtils";
const FTPDeploy = require("ftp-deploy");

export interface UploadOptions {
  user: string;
  password: string;
  host: string;
  port: number;
  localRoot: string;
  remoteRoot: string;
  include: string[];
  exclude: string[];
  deleteRemote: boolean;
  forcePasv: boolean;
  sftp: boolean;
  isRootRelative: boolean;
}

export async function run(args?: string) {
  const ftpDeploy = new FTPDeploy();
  let workspaceFolder: WorkspaceFolder;
  let fileToUpload: string;

  // If we receive arguments, the file to upload has already been figured out for us,
  // else, we use the user's choice, main compilation file or current editor
  if (!args) {
    workspaceFolder = workspace.getWorkspaceFolder(lastActiveEditor.document.uri);
    const compileMainPath: boolean = getConfig(Section.SourcePawn, "MainPathCompilation", workspaceFolder);
    if (compileMainPath) {
      fileToUpload = await getMainCompilationFile();
    }
    else {
      fileToUpload = lastActiveEditor.document.uri.fsPath
    }
  }
  else {
    fileToUpload = args;
    workspaceFolder = workspace.getWorkspaceFolder(URI.file(fileToUpload));
  }

  // Return if upload settings are not defined
  const uploadOptions: UploadOptions = getConfig(Section.SourcePawn, "UploadOptions", workspaceFolder)
  if (uploadOptions === undefined) {
    window.showErrorMessage("Upload settings are empty.", "Open Settings")
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
  if (uploadOptions.user == "" || uploadOptions.host == "") {
    window.showErrorMessage("Cannot upload - user or host empty.", "Open Settings")
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
  uploadOptions.deleteRemote = false;

  // If specified, replace macro with main compilation file
  uploadOptions.localRoot = uploadOptions.localRoot.replace("${mainPath}", await getMainCompilationFile())

  if (uploadOptions.isRootRelative) {
    // Concat the workspace with it's root if the path is relative.
    if (workspaceFolder === undefined) {
      window.showWarningMessage(
        "No workspace or folder found, with isRootRelative is set to true.\nSet it to false, or open the file from a workspace."
      );
      return 1;
    }
    const workspaceRoot = workspaceFolder.uri.fsPath;
    uploadOptions.localRoot = join(workspaceRoot, uploadOptions.localRoot);
  }

  // Copy the config object to avoid https://github.com/microsoft/vscode/issues/80976
  const ftpConfig = { ...uploadOptions };
  // Delete that setting to avoid problems with the ftp/sftp library
  delete ftpConfig.isRootRelative;

  console.log("Starting the upload");
  console.log(ftpConfig);
  ftpDeploy
    .deploy(ftpConfig)
    .then(() => {
      console.log("Upload is finished.");
      const commandsOption: string = getConfig(Section.SourcePawn, "runServerCommands", workspaceFolder);
      if (commandsOption === "afterUpload") {
        runServerCommands(fileToUpload);
      }
    })
    .catch(err => {
      // TODO: inform the user
      return console.error(err);
    });
  return 0;
}
