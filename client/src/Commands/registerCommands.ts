import { commands, ExtensionContext } from "vscode";
import * as CreateTaskCommand from "./createTask";
import * as CreateScriptCommand from "./createScript";
import * as CreateREADMECommand from "./createREADME";
import * as CreateMasterCommand from "./createGitHubActions";
import * as CreateProjectCommand from "./createProject";
import * as CompileSMCommand from "./compileSM";
import * as UploadToServerCommand from "./uploadToServer";
import * as RefreshPluginsCommand from "./refreshPlugins";

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
}
