import vscode = require("vscode");
import * as fs from "fs";
import * as path from "path";
import CreateTaskCommand = require("./createTask");
import CreateScriptCommand = require("./createScript");
import CreateREADMECommand = require("./createREADME");
import CreateMasterCommand = require("./createGitHubActions");

export async function run(args: any) {
  // get workspace folder
  let workspaceFolders = vscode.workspace.workspaceFolders;
  if (!workspaceFolders) {
    vscode.window.showErrorMessage("No workspace are opened.");
    return;
  }

	const inputOptions: vscode.InputBoxOptions = {
		prompt: "Relative path for the root of the project. Leave empty for the root path."
	}

	const input = await vscode.window.showInputBox(inputOptions);

  //Select the rootpath
	let rootpath = path.join(workspaceFolders?.[0].uri.fsPath, input);
	if (!fs.existsSync(rootpath)){
		fs.mkdirSync(rootpath);
  }

  // Create the plugins folder
  let pluginsFolderPath = path.join(rootpath, "plugins");
  if (!fs.existsSync(pluginsFolderPath)) {
    fs.mkdirSync(pluginsFolderPath);
  }

  // Running the other commands
  CreateTaskCommand.run(rootpath);
  CreateScriptCommand.run(rootpath);
  CreateREADMECommand.run(rootpath);
  CreateMasterCommand.run(rootpath);
}
