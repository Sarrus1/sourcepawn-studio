import { commands, ExtensionContext } from "vscode";
import { run as CreateTaskCommand } from "./createTask";
import { run as CreateScriptCommand } from "./createScript";
import { run as CreateREADMECommand } from "./createREADME";
import { run as CreateMasterCommand } from "./createGitHubActions";
import { run as CreateProjectCommand } from "./createProject";
import { run as CompileSMCommand } from "./compileSM";
import { run as UploadToServerCommand } from "./uploadToServer";
import { run as RefreshPluginsCommand } from "./refreshPlugins";
import { run as InsertParametersCommand } from "./insertParameters";
import { run as setFileAsMainCommand } from "./setFileAsMain";
import { run as installSMCommand } from "./installSM";
import { run as createChangelogCommand } from "./createCHANGELOG";
import { run as changeSMApiCommand } from "./changeSMApi";

/**
 * Register all the commands of the extension.
 * @param  {ExtensionContext} context The extension's context.
 * @returns void
 */
export function registerSMCommands(context: ExtensionContext): void {
  let createTask = commands.registerCommand(
    "sourcepawn-vscode.createTask",
    CreateTaskCommand.bind(undefined)
  );
  context.subscriptions.push(createTask);

  let createScript = commands.registerCommand(
    "sourcepawn-vscode.createScript",
    CreateScriptCommand.bind(undefined)
  );
  context.subscriptions.push(createScript);

  let createREADME = commands.registerCommand(
    "sourcepawn-vscode.createREADME",
    CreateREADMECommand.bind(undefined)
  );
  context.subscriptions.push(createREADME);

  let createMaster = commands.registerCommand(
    "sourcepawn-vscode.createMaster",
    CreateMasterCommand.bind(undefined)
  );
  context.subscriptions.push(createMaster);

  let createProject = commands.registerCommand(
    "sourcepawn-vscode.createProject",
    CreateProjectCommand.bind(undefined)
  );
  context.subscriptions.push(createProject);

  let compileSM = commands.registerCommand(
    "sourcepawn-vscode.compileSM",
    CompileSMCommand.bind(undefined)
  );
  context.subscriptions.push(compileSM);

  let uploadToServer = commands.registerCommand(
    "sourcepawn-vscode.uploadToServer",
    UploadToServerCommand.bind(undefined)
  );
  context.subscriptions.push(uploadToServer);

  let refreshPlugins = commands.registerCommand(
    "sourcepawn-vscode.refreshPlugins",
    RefreshPluginsCommand.bind(undefined)
  );
  context.subscriptions.push(refreshPlugins);

  let insertParameters = commands.registerCommand(
    "sourcepawn-vscode.insertParameters",
    InsertParametersCommand.bind(undefined)
  );
  context.subscriptions.push(insertParameters);

  let setFileAsMain = commands.registerCommand(
    "sourcepawn-vscode.setFileAsMain",
    setFileAsMainCommand.bind(undefined)
  );
  context.subscriptions.push(setFileAsMain);

  let installSM = commands.registerCommand(
    "sourcepawn-vscode.installSM",
    installSMCommand.bind(undefined)
  );
  context.subscriptions.push(installSM);

  let createChangelog = commands.registerCommand(
    "sourcepawn-vscode.createChangelog",
    createChangelogCommand.bind(undefined)
  );
  context.subscriptions.push(createChangelog);

  let changeSMApi = commands.registerCommand(
    "sourcepawn-vscode.changeSMApi",
    changeSMApiCommand.bind(undefined)
  );
  context.subscriptions.push(changeSMApi);
}
