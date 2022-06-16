import { workspace as Workspace, window, commands } from "vscode";
import { join } from "path";
import { run as refreshPluginsCommand } from "./refreshPlugins";
import { findMainPath } from "../spUtils";
// Keep the include like this,
// otherwise FTPDeploy is not
// recognised as a constructor
const FTPDeploy = require("ftp-deploy");

export async function run(args: any) {
  const ftpDeploy = new FTPDeploy();
  const workspaceFolder =
    args === undefined ? undefined : Workspace.getWorkspaceFolder(args);
  const config: object = Workspace.getConfiguration(
    "sourcepawn",
    workspaceFolder
  ).get("UploadOptions");
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

  if (config["localRoot"] === "${mainPath}") {
    config["localRoot"] = findMainPath();
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
  ftpDeploy
    .deploy(ftpConfig)
    .then(() => {
      console.log("Upload is finished.");
      if (
        Workspace.getConfiguration("sourcepawn", workspaceFolder).get<string>(
          "refreshServerPlugins"
        ) === "afterUpload"
      ) {
        refreshPluginsCommand(undefined);
      }
    })
    .catch((err) => console.error(err));
  return 0;
}
