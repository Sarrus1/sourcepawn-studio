import { workspace as Workspace, window, InputBoxOptions } from "vscode";
import { existsSync, mkdirSync } from "fs";
import { join } from "path";
import { run as createTaskCommand } from "./createTask";
import { run as createScriptCommand } from "./createScript";
import { run as createREADMECommand } from "./createREADME";
import { run as createMasterCommand } from "./createGitHubActions";
import { run as createChangelogCommand } from "./createCHANGELOG";
import { run as createLicenseCommand } from "./createLICENSE";
import { run as createGitignoreCommand } from "./createGITIGNORE";

export async function run(args: any) {
  // Get workspace folder
  const workspaceFolders = Workspace.workspaceFolders;
  if (!workspaceFolders) {
    window.showErrorMessage("No workspaces are opened.");
    return;
  }

  const inputOptions: InputBoxOptions = {
    prompt:
      "Relative path for the root of the project. Leave empty for the root ",
  };

  const input = await window.showInputBox(inputOptions);

  //Select the rootpath
  const rootpath = join(workspaceFolders?.[0].uri.fsPath, input);
  if (!existsSync(rootpath)) {
    mkdirSync(rootpath);
  }

  // Create the plugins folder
  const pluginsFolderPath = join(rootpath, "plugins");
  if (!existsSync(pluginsFolderPath)) {
    mkdirSync(pluginsFolderPath);
  }

  // Running the other commands
  createTaskCommand(rootpath);
  createScriptCommand(rootpath);
  createREADMECommand(rootpath);
  createMasterCommand(rootpath);
  createChangelogCommand(rootpath);
  createLicenseCommand(rootpath);
  createGitignoreCommand(rootpath);
}
