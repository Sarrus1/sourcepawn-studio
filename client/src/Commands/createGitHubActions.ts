import vscode = require("vscode");
import * as fs from "fs";
import * as path from 'path';

export function run(args: any) {

    // get workspace folder
    let workspaceFolders = vscode.workspace.workspaceFolders;
    if (!workspaceFolders) {
        vscode.window.showErrorMessage("No workspace are opened.");
        return 1;
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
			return 1;
		}
		let myExtDir : string = vscode.extensions.getExtension ("Sarrus.sourcepawn-vscode").extensionPath;
		let tasksTemplatesPath : string = path.join(myExtDir, "templates/master_template.yml");
		fs.copyFileSync(tasksTemplatesPath, masterFilePath);

		// Replace placeholders
		try{
			let result = fs.readFileSync(masterFilePath, 'utf8');
			result = result.replace(/\${plugin_name}/gm, rootname);
			fs.writeFileSync(masterFilePath, result, 'utf8');
		}
		catch(err){
			console.log(err);
			return 1;
		}
	return 0;
}