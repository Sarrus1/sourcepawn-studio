import * as vscode from "vscode";
import * as CreateTaskCommand from "./createTask";
import * as CreateScriptCommand from "./createScript";
import * as CreateREADMECommand from "./createREADME";
import * as CreateMasterCommand from "./createGitHubActions";
import * as CreateProjectCommand from "./createProject";
import * as CompileSMCommand from "./compileSM";
	
export function registerSMCommands (context : vscode.ExtensionContext){
	let createTask = vscode.commands.registerCommand(
    "extension.createTask",
    CreateTaskCommand.run.bind(undefined)
  );
  context.subscriptions.push(createTask);

  let createScript = vscode.commands.registerCommand(
    "extension.createScript",
    CreateScriptCommand.run.bind(undefined)
  );
  context.subscriptions.push(createScript);

  let createREADME = vscode.commands.registerCommand(
    "extension.createREADME",
    CreateREADMECommand.run.bind(undefined)
  );
  context.subscriptions.push(createREADME);

  let createMaster = vscode.commands.registerCommand(
    "extension.createMaster",
    CreateMasterCommand.run.bind(undefined)
  );
  context.subscriptions.push(createMaster);

  let createProject = vscode.commands.registerCommand(
    "extension.createProject",
    CreateProjectCommand.run.bind(undefined)
  );
  context.subscriptions.push(createProject);

	let compileSM = vscode.commands.registerCommand(
    "extension.compileSM",
    CompileSMCommand.run.bind(undefined)
  );
  context.subscriptions.push(compileSM);
}
  