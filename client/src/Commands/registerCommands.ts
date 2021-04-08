import {commands, ExtensionContext} from "vscode";
import * as CreateTaskCommand from "./createTask";
import * as CreateScriptCommand from "./createScript";
import * as CreateREADMECommand from "./createREADME";
import * as CreateMasterCommand from "./createGitHubActions";
import * as CreateProjectCommand from "./createProject";
import * as CompileSMCommand from "./compileSM";
	
export function registerSMCommands (context : ExtensionContext){
	let createTask = commands.registerCommand(
    "extension.createTask",
    CreateTaskCommand.run.bind(undefined)
  );
  context.subscriptions.push(createTask);

  let createScript = commands.registerCommand(
    "extension.createScript",
    CreateScriptCommand.run.bind(undefined)
  );
  context.subscriptions.push(createScript);

  let createREADME = commands.registerCommand(
    "extension.createREADME",
    CreateREADMECommand.run.bind(undefined)
  );
  context.subscriptions.push(createREADME);

  let createMaster = commands.registerCommand(
    "extension.createMaster",
    CreateMasterCommand.run.bind(undefined)
  );
  context.subscriptions.push(createMaster);

  let createProject = commands.registerCommand(
    "extension.createProject",
    CreateProjectCommand.run.bind(undefined)
  );
  context.subscriptions.push(createProject);

	let compileSM = commands.registerCommand(
    "extension.compileSM",
    CompileSMCommand.run.bind(undefined)
  );
  context.subscriptions.push(compileSM);
}
  