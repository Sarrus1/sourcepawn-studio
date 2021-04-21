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

  //Select the rootpath
  let rootpath = workspaceFolders?.[0].uri;
  let rootname = workspaceFolders?.[0].name;

  // Create the plugins folder
  let pluginsFolderPath = path.join(rootpath.fsPath, "plugins");
  if (!fs.existsSync(pluginsFolderPath)) {
    fs.mkdirSync(pluginsFolderPath);
  }

  // Running the other commands
  CreateTaskCommand.run(undefined);
  CreateScriptCommand.run(undefined);
  CreateREADMECommand.run(undefined);
  CreateMasterCommand.run(undefined);
}
