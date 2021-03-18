import vscode = require("vscode");
import * as fs from "fs";
import * as path from 'path';

export async function run(args: any) {

		let author_name : string = vscode.workspace.getConfiguration("sourcepawnLanguageServer").get(
			"author_name"
		)
		if(!author_name){
			vscode.window
			.showWarningMessage(
				"You didn't specify an author name.",
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

		let github_name : string = vscode.workspace.getConfiguration("sourcepawnLanguageServer").get(
			"github_name"
		)

    // get workspace folder
    let workspaceFolders = vscode.workspace.workspaceFolders;
    if (!workspaceFolders) {
        vscode.window.showErrorMessage("No workspace are opened.");
        return;
    }

		//Select the rootpath
		let rootpath = workspaceFolders?.[0].uri;
		let rootname = workspaceFolders?.[0].name;

		// create a scripting folder if it doesn't exist
		let scriptingFolderPath = path.join(rootpath.fsPath, "scripting");
		if (!fs.existsSync(scriptingFolderPath)){
			fs.mkdirSync(scriptingFolderPath);
		}

		// Check if file already exists
		let scriptFileName:string = rootname + ".sp";
		let scriptFilePath = path.join(rootpath.fsPath, "scripting", scriptFileName);
		if (fs.existsSync(scriptFilePath)){
			vscode.window.showErrorMessage(scriptFileName+ " already exists, aborting.");
			return;
		}
		let myExtDir : string = vscode.extensions.getExtension ("Sarrus.sourcepawn-vscode").extensionPath;
		let tasksTemplatesPath : string = path.join(myExtDir, "templates/plugin_template.sp");
		fs.copyFile(tasksTemplatesPath, scriptFilePath, (err) => {
			if (err){
				vscode.window.showErrorMessage("An error has occured.");
				throw err;
			} 
		});

		// Replace placeholders
		fs.readFile(scriptFilePath, 'utf8', function (err,data) {
			if (err) {
				return console.log(err);
			}
			let result = data.replace(/\${author_name}/gm, author_name);
			result = result.replace(/\${plugin_name}/gm, rootname);
			result = result.replace(/\${github_name}/gm, github_name);
			fs.writeFile(scriptFilePath, result, 'utf8', function (err) {
				 if (err) return console.log(err);
			});
		});
}