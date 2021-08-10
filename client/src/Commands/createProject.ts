import { workspace as Workspace, window, InputBoxOptions } from "vscode";
import { existsSync, mkdirSync } from "fs";
import { join } from "path";
import { run as CreateTaskCommand } from "./createTask";
import { run as CreateScriptCommand } from "./createScript";
import { run as CreateREADMECommand } from "./createREADME";
import { run as CreateMasterCommand } from "./createGitHubActions";

export async function run(args: any) {
  // get workspace folder
  let workspaceFolders = Workspace.workspaceFolders;
  if (!workspaceFolders) {
    window.showErrorMessage("No workspace are opened.");
    return;
  }

  const inputOptions: InputBoxOptions = {
    prompt:
      "Relative path for the root of the project. Leave empty for the root ",
  };

  const input = await window.showInputBox(inputOptions);

  //Select the rootpath
  let rootpath = join(workspaceFolders?.[0].uri.fsPath, input);
  if (!existsSync(rootpath)) {
    mkdirSync(rootpath);
  }

  // Create the plugins folder
  let pluginsFolderPath = join(rootpath, "plugins");
  if (!existsSync(pluginsFolderPath)) {
    mkdirSync(pluginsFolderPath);
  }

  // Running the other commands
  CreateTaskCommand(rootpath);
  CreateScriptCommand(rootpath);
  CreateREADMECommand(rootpath);
  CreateMasterCommand(rootpath);
}
