import vscode = require("vscode");
import * as fs from "fs";
import * as path from 'path';

export async function run(args: any) {

    // get workspace folder
    let workspaceFolders = vscode.workspace.workspaceFolders;
    if (!workspaceFolders) {
        vscode.window.showErrorMessage("No workspace are opened.");
        return;
    }

		//Select the rootpath
		let rootpath = workspaceFolders?.[0].uri;
		let rootname = workspaceFolders?.[0].name;

		// create .github folder if it doesn't exist
		let masterFolderPath = path.join(rootpath.fsPath, ".github");
		if (!fs.existsSync(masterFolderPath)){
			fs.mkdirSync(masterFolderPath);
		}
		// create workflows folder if it doesn't exist
		masterFolderPath = path.join(rootpath.fsPath, ".github", "workflows");
		if (!fs.existsSync(masterFolderPath)){
			fs.mkdirSync(masterFolderPath);
		}

		// Check if master.yml already exists
		let masterFilePath = path.join(rootpath.fsPath, ".github/workflows/master.yml");
		if (fs.existsSync(masterFilePath)){
			vscode.window.showErrorMessage("master.yml already exists, aborting.");
			return;
		}
		let myExtDir : string = vscode.extensions.getExtension ("Sarrus.sourcepawn-vscode").extensionPath;
		let tasksTemplatesPath : string = path.join(myExtDir, "templates/master_template.yml");
		fs.copyFile(tasksTemplatesPath, masterFilePath, (err) => {
			if (err){
				vscode.window.showErrorMessage("An error has occured.");
				throw err;
			} 
		});

		// Replace placeholders
		fs.readFile(masterFilePath, 'utf8', function (err,data) {
			if (err) {
				return console.log(err);
			}
			let result = data.replace(/\${plugin_name}/gm, rootname);
			fs.writeFile(masterFilePath, result, 'utf8', function (err) {
				 if (err) return console.log(err);
			});
		});
}