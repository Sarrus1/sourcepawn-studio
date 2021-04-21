import vscode = require("vscode");
import {join} from "path";
// Keep the include like this,
// otherwise FTPDeploy is not
// recognised as a constructor
const FTPDeploy = require("ftp-deploy");

export async function run(args: any) {
  let ftpDeploy = new FTPDeploy();
  let config: object = vscode.workspace
    .getConfiguration("sourcepawn")
    .get("UploadOptions");
  if (typeof config == "undefined") {
    vscode.window
      .showErrorMessage("Upload settings are empty.", "Open Settings")
      .then((choice) => {
        if (choice === "Open Settings") {
          vscode.commands.executeCommand(
            "workbench.action.openWorkspaceSettings"
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
            "workbench.action.openWorkspaceSettings"
          );
        }
      });
    return 2;
  }
	// Override the "deleteRemote" setting for safety.
	config["deleteRemote"] = false;
	// Make sure the path to upload is relative to avoid uploading the whole disk.
	let workspaceRoot : string = vscode.workspace.workspaceFolders[0].uri.fsPath;
	config["localRoot"] = join(workspaceRoot, config["localRoot"]);
  ftpDeploy
    .deploy(config)
    .then(async (res) => {
      console.log("Upload is finished.");
      if (
        vscode.workspace
          .getConfiguration("sourcepawn")
          .get("uploadAfterSuccessfulCompile")
      ) {
        await vscode.commands.executeCommand(
          "sourcepawn-vscode.refreshPlugins"
        );
      }
    })
    .catch((err) => console.error(err));
}
