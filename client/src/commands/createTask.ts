import vscode = require("vscode");
import * as fs from "fs";
import * as path from 'path';

export async function run(args: any) {

		// Get configuration
		let sm_home : string = vscode.workspace.getConfiguration("sourcepawnLanguageServer").get(
			"sourcemod_home"
		)
		if(!sm_home){
			vscode.window
			.showWarningMessage(
				"SourceMod API not found in the project. You should set SourceMod Home for tasks generation to work.",
				"Open Settings"
			)
			.then((choice) => {
				if (choice === "Open Settings") {
					vscode.commands.executeCommand(
						"workbench.action.openWorkspaceSettings"
					);
				}
			});
		}

		let spcomp_path : string = vscode.workspace.getConfiguration("sourcepawnLanguageServer").get(
			"spcomp_path"
		)
		if(!spcomp_path){
			vscode.window
			.showErrorMessage(
				"SourceMod compiler not found in the project. You need to set spcomp path for tasks generation to work.",
				"Open Settings"
			)
			.then((choice) => {
				if (choice === "Open Settings") {
					vscode.commands.executeCommand(
						"workbench.action.openWorkspaceSettings"
					);
				}
			});
			return;
		}

    // get workspace folder
    let workspaceFolders = vscode.workspace.workspaceFolders;
    if (!workspaceFolders) {
        vscode.window.showErrorMessage("No workspace are opened.");
        return;
    }

		//Select the rootpath
		let rootpath = workspaceFolders?.[0].uri;

		// create task folder if it doesn't exist
		let taskFolderPath = path.join(rootpath.fsPath, ".vscode");
		if (!fs.existsSync(taskFolderPath)){
			fs.mkdirSync(taskFolderPath);
		}

		// Check if file already exists
		let taskFilePath = path.join(rootpath.fsPath, ".vscode/tasks.json");
		if (fs.existsSync(taskFilePath)){
			vscode.window.showErrorMessage("Tasks file already exists.");
			return;
		}
		let myExtDir : string = vscode.extensions.getExtension ("Sarrus.sourcepawn-vscode").extensionPath;
		let tasksTemplatesPath : string = path.join(myExtDir, "templates/tasks.json");
		fs.copyFile(tasksTemplatesPath, taskFilePath, (err) => {
			if (err){
				vscode.window.showErrorMessage("An error has occured.");
				throw err;
			} 
			vscode.window.showInformationMessage("The tasks file was generated.");
		});

		// Replace placeholders
		fs.readFile(taskFilePath, 'utf8', function (err,data) {
			if (err) {
				return console.log(err);
			}
			let result = data.replace(/\${spcomp_path}/gm, spcomp_path);
			result = result.replace(/\${include_path}/gm, sm_home);
			fs.writeFile(taskFilePath, result, 'utf8', function (err) {
				 if (err) return console.log(err);
			});
		});
}