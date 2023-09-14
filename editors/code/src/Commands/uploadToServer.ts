import * as vscode from "vscode";
import { join } from "path";

import { run as refreshPluginsCommand } from "./refreshPlugins";
import { ctx } from "../spIndex";
import { ProjectMainPathParams, projectMainPath } from "../lsp_ext";
import { URI } from "vscode-uri";
const FTPDeploy = require("ftp-deploy");

export async function run(args: any) {
  const ftpDeploy = new FTPDeploy();
  const workspaceFolder =
    args === undefined ? undefined : vscode.workspace.getWorkspaceFolder(args);
  const config: object = vscode.workspace
    .getConfiguration("sourcepawn", workspaceFolder)
    .get("UploadOptions");
  if (config === undefined) {
    vscode.window
      .showErrorMessage("Upload settings are empty.", "Open Settings")
      .then((choice) => {
        if (choice === "Open Settings") {
          vscode.commands.executeCommand(
            "workbench.action.openSettings",
            "@ext:sarrus.sourcepawn-vscode"
          );
        }
      });
    return 1;
  }
  if (config["user"] == "" || config["host"] == "") {
    vscode.window
      .showErrorMessage(
        "Some settings are improperly defined in the upload settings.",
        "Open Settings"
      )
      .then((choice) => {
        if (choice === "Open Settings") {
          vscode.commands.executeCommand(
            "workbench.action.openSettings",
            "@ext:sarrus.sourcepawn-vscode"
          );
        }
      });
    return 2;
  }
  // Override the "deleteRemote" setting for safety.
  config["deleteRemote"] = false;

  if (config["localRoot"] === "${mainPath}") {
    const params: ProjectMainPathParams = {
      uri: vscode.window.activeTextEditor.document.uri.toString(),
    };
    const mainUri = await ctx?.client.sendRequest(projectMainPath, params);
    config["localRoot"] = URI.parse(mainUri).fsPath;
  }

  if (config["isRootRelative"]) {
    // Concat the workspace with it's root if the path is relative.
    if (workspaceFolder === undefined) {
      vscode.window.showWarningMessage(
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
        vscode.workspace
          .getConfiguration("sourcepawn", workspaceFolder)
          .get<string>("refreshServerPlugins") === "afterUpload"
      ) {
        refreshPluginsCommand(undefined);
      }
    })
    .catch((err) => console.error(err));
  return 0;
}
