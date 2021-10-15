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

export function registerSMCommands(context: ExtensionContext) {
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

  let UploadToServer = commands.registerCommand(
    "sourcepawn-vscode.uploadToServer",
    UploadToServerCommand.bind(undefined)
  );
  context.subscriptions.push(UploadToServer);

  let RefreshPlugins = commands.registerCommand(
    "sourcepawn-vscode.refreshPlugins",
    RefreshPluginsCommand.bind(undefined)
  );
  context.subscriptions.push(RefreshPlugins);

  let InsertParameters = commands.registerTextEditorCommand(
    "sourcepawn-vscode.insertParameters",
    InsertParametersCommand.bind(undefined)
  );
  context.subscriptions.push(InsertParameters);

  let setFileAsMain = commands.registerTextEditorCommand(
    "sourcepawn-vscode.setFileAsMain",
    setFileAsMainCommand.bind(undefined)
  );
  context.subscriptions.push(setFileAsMain);

  let installSM = commands.registerTextEditorCommand(
    "sourcepawn-vscode.installSM",
    installSMCommand.bind(undefined)
  );
  context.subscriptions.push(installSM);

  let createChangelog = commands.registerTextEditorCommand(
    "sourcepawn-vscode.createChangelog",
    createChangelogCommand.bind(undefined)
  );
  context.subscriptions.push(createChangelog);
}
