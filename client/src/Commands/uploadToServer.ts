import { workspace as Workspace, window, commands } from "vscode";
import { join } from "path";
import { run as refreshPluginsCommand } from "./refreshPlugins";
// Keep the include like this,
// otherwise FTPDeploy is not
// recognised as a constructor
const FTPDeploy = require("ftp-deploy");

export async function run(args: any) {
  let ftpDeploy = new FTPDeploy();
  let workspaceFolder = Workspace.getWorkspaceFolder(args.document.uri);
  let config: object = Workspace.getConfiguration(
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

  // Make sure the path to upload is relative to avoid uploading the whole disk.
  let workspaceFolders = Workspace.workspaceFolders;
  if (workspaceFolders === undefined) {
    window.showWarningMessage(
      "No workspace or folder found. \n Please open the folder containing your .sp file, not just the .sp file."
    );
    return 1;
  }
  let workspaceRoot: string = workspaceFolders[0].uri.fsPath;
  config["localRoot"] = join(workspaceRoot, config["localRoot"]);
  ftpDeploy
    .deploy(config)
    .then(() => {
      console.log("Upload is finished.");
      if (
        Workspace.getConfiguration("sourcepawn", workspaceFolder).get(
          "uploadAfterSuccessfulCompile"
        )
      ) {
        refreshPluginsCommand(undefined);
      }
    })
    .catch((err) => console.error(err));
  return 0;
}
