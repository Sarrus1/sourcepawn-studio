import { commands, ExtensionContext } from "vscode";
import * as CreateTaskCommand from "./createTask";
import * as CreateScriptCommand from "./createScript";
import * as CreateREADMECommand from "./createREADME";
import * as CreateMasterCommand from "./createGitHubActions";
import * as CreateProjectCommand from "./createProject";
import * as CompileSMCommand from "./compileSM";
import * as UploadToServerCommand from "./uploadToServer";
import * as RefreshPluginsCommand from "./refreshPlugins";
import * as InsertParametersCommand from "./insertParameters";
import * as setFileAsMainCommand from "./setFileAsMain";
import * as installSMCommand from "./installSM";

export function registerSMCommands(context: ExtensionContext) {
  let createTask = commands.registerCommand(
    "sourcepawn-vscode.createTask",
    CreateTaskCommand.run.bind(undefined)
  );
  context.subscriptions.push(createTask);

  let createScript = commands.registerCommand(
    "sourcepawn-vscode.createScript",
    CreateScriptCommand.run.bind(undefined)
  );
  context.subscriptions.push(createScript);

  let createREADME = commands.registerCommand(
    "sourcepawn-vscode.createREADME",
    CreateREADMECommand.run.bind(undefined)
  );
  context.subscriptions.push(createREADME);

  let createMaster = commands.registerCommand(
    "sourcepawn-vscode.createMaster",
    CreateMasterCommand.run.bind(undefined)
  );
  context.subscriptions.push(createMaster);

  let createProject = commands.registerCommand(
    "sourcepawn-vscode.createProject",
    CreateProjectCommand.run.bind(undefined)
  );
  context.subscriptions.push(createProject);

  let compileSM = commands.registerCommand(
    "sourcepawn-vscode.compileSM",
    CompileSMCommand.run.bind(undefined)
  );
  context.subscriptions.push(compileSM);

  let UploadToServer = commands.registerCommand(
    "sourcepawn-vscode.uploadToServer",
    UploadToServerCommand.run.bind(undefined)
  );
  context.subscriptions.push(UploadToServer);

  let RefreshPlugins = commands.registerCommand(
    "sourcepawn-vscode.refreshPlugins",
    RefreshPluginsCommand.run.bind(undefined)
  );
  context.subscriptions.push(RefreshPlugins);

  let InsertParameters = commands.registerTextEditorCommand(
    "sourcepawn-vscode.insertParameters",
    InsertParametersCommand.run.bind(undefined)
  );
  context.subscriptions.push(InsertParameters);

  let setFileAsMain = commands.registerTextEditorCommand(
    "sourcepawn-vscode.setFileAsMain",
    setFileAsMainCommand.run.bind(undefined)
  );
  context.subscriptions.push(setFileAsMain);

	let installSM = commands.registerTextEditorCommand(
    "sourcepawn-vscode.installSM",
    installSMCommand.run.bind(undefined)
  );
  context.subscriptions.push(installSM);
	
}
