import * as vscode from "vscode";
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
import { run as createGitignoreCommand } from "./createGITIGNORE";
import { run as createLicenseCommand } from "./createLICENSE";
import { run as changeSMApiCommand } from "./changeSMApi";
import { run as installLanguageServerCommand } from "./installLanguageServer";
import { run as doctorCommand } from "./doctor";
import { preprocessedDocumentCommand } from "./preprocessedDocument";
import { CommandFactory } from "../ctx";

/**
 * Register all the vscode.commands of the extension.
 * @param  {vscode.ExtensionContext} context The extension's context.
 * @returns void
 */
export function registerSMCommands(context: vscode.ExtensionContext): void {
  const createTask = vscode.commands.registerCommand(
    "sourcepawn-vscode.createTask",
    CreateTaskCommand.bind(undefined)
  );
  context.subscriptions.push(createTask);

  const createScript = vscode.commands.registerCommand(
    "sourcepawn-vscode.createScript",
    CreateScriptCommand.bind(undefined)
  );
  context.subscriptions.push(createScript);

  const createREADME = vscode.commands.registerCommand(
    "sourcepawn-vscode.createREADME",
    CreateREADMECommand.bind(undefined)
  );
  context.subscriptions.push(createREADME);

  const createMaster = vscode.commands.registerCommand(
    "sourcepawn-vscode.createMaster",
    CreateMasterCommand.bind(undefined)
  );
  context.subscriptions.push(createMaster);

  const createProject = vscode.commands.registerCommand(
    "sourcepawn-vscode.createProject",
    CreateProjectCommand.bind(undefined)
  );
  context.subscriptions.push(createProject);

  const compileSM = vscode.commands.registerCommand(
    "sourcepawn-vscode.compileSM",
    CompileSMCommand.bind(undefined)
  );
  context.subscriptions.push(compileSM);

  const uploadToServer = vscode.commands.registerCommand(
    "sourcepawn-vscode.uploadToServer",
    UploadToServerCommand.bind(undefined)
  );
  context.subscriptions.push(uploadToServer);

  const refreshPlugins = vscode.commands.registerCommand(
    "sourcepawn-vscode.refreshPlugins",
    RefreshPluginsCommand.bind(undefined)
  );
  context.subscriptions.push(refreshPlugins);

  const insertParameters = vscode.commands.registerCommand(
    "sourcepawn-vscode.insertParameters",
    InsertParametersCommand.bind(undefined)
  );
  context.subscriptions.push(insertParameters);

  const setFileAsMain = vscode.commands.registerCommand(
    "sourcepawn-vscode.setFileAsMain",
    setFileAsMainCommand.bind(undefined)
  );
  context.subscriptions.push(setFileAsMain);

  const installSM = vscode.commands.registerCommand(
    "sourcepawn-vscode.installSM",
    installSMCommand.bind(undefined)
  );
  context.subscriptions.push(installSM);

  const createChangelog = vscode.commands.registerCommand(
    "sourcepawn-vscode.createChangelog",
    createChangelogCommand.bind(undefined)
  );
  context.subscriptions.push(createChangelog);

  const createGitignore = vscode.commands.registerCommand(
    "sourcepawn-vscode.createGitignore",
    createGitignoreCommand.bind(undefined)
  );
  context.subscriptions.push(createGitignore);

  const createLicense = vscode.commands.registerCommand(
    "sourcepawn-vscode.createLicense",
    createLicenseCommand.bind(undefined)
  );
  context.subscriptions.push(createLicense);

  const changeSMApi = vscode.commands.registerCommand(
    "sourcepawn-vscode.changeSMApi",
    changeSMApiCommand.bind(undefined)
  );
  context.subscriptions.push(changeSMApi);

  const installLanguageServer = vscode.commands.registerCommand(
    "sourcepawn-vscode.installLanguageServer",
    installLanguageServerCommand.bind(undefined)
  );
  context.subscriptions.push(installLanguageServer);

  const doctor = vscode.commands.registerCommand(
    "sourcepawn-vscode.doctor",
    doctorCommand.bind(undefined)
  );
  context.subscriptions.push(doctor);
}

/**
 * Prepare a record of server specific commands.
 * @returns Record
 */
export function createServerCommands(): Record<string, CommandFactory> {
  return {
    startServer: {
      enabled: (ctx) => async () => {
        await ctx.restart();
      },
      disabled: (ctx) => async () => {
        await ctx.start();
      },
    },
    stopServer: {
      enabled: (ctx) => async () => {
        await ctx.stopAndDispose();
        ctx.setServerStatus({
          health: "stopped",
        });
      },
      disabled: (_) => async () => {},
    },
    openLogs: {
      enabled: (ctx) => async () => {
        if (ctx.client.outputChannel) {
          ctx.client.outputChannel.show();
        }
      },
      disabled: (_) => async () => {},
    },
    preprocessedDocument: {
      enabled: preprocessedDocumentCommand,
    },
  };
}
