import vscode = require("vscode");
import * as fs from "fs";
import * as path from 'path';
// Keep the include like this, 
// otherwise FTPDeploy is not 
// recognised as a constructor
const FTPDeploy = require('ftp-deploy')

export async function run(args: any) {
	let ftpDeploy = new FTPDeploy();
	let config:object = vscode.workspace.getConfiguration("sourcepawn").get("UploadOptions");
	if(typeof config == "undefined")
	{
		vscode.window
		.showErrorMessage(
			"Upload settings are empty.",
			"Open Settings"
		).then((choice) => {
			if (choice === "Open Settings") {
				vscode.commands.executeCommand(
					"workbench.action.openWorkspaceSettings"
				);
			}
		});
		return 1;
	}
	if(config["user"] == "" || config["host"] == ""){
		vscode.window
		.showErrorMessage(
			"Some settings are improperly defined in the upload settings.",
			"Open Settings"
		).then((choice) => {
			if (choice === "Open Settings") {
				vscode.commands.executeCommand(
					"workbench.action.openWorkspaceSettings"
				);
			}
		});
		return 2;
	}
	ftpDeploy
	.deploy(config)
	.then(
		res => console.log("Upload is finished.")
		)
	.catch(err => console.error(err));
}