import { workspace as workspace, window, commands, WorkspaceFolder } from "vscode";
import Rcon from "rcon-srcds";
import { EncodingOptions } from "rcon-srcds/dist/packet";
import { getMainCompilationFile, getPluginName } from "../spUtils";
import { lastActiveEditor } from "../spIndex";
import { URI } from "vscode-uri";
import { Section, getConfig } from "../configUtils";

export interface ServerOptions {
  host: string;
  password: string;
  port: number;
  encoding: EncodingOptions;
  timeout: number;
}

/**
 * Callback function for Run Server Commands command.
 * @param args URI of the plugin that has been compiled.
 * @returns A Promise.
 */
export async function run(args?: string) {
  let workspaceFolder: WorkspaceFolder;

  // If we don't receive args, we need to figure out which plugin was sent
  if (!args) {
    workspaceFolder = workspace.getWorkspaceFolder(lastActiveEditor.document.uri);
    const compileMainPath: boolean = getConfig(Section.SourcePawn, "MainPathCompilation", workspaceFolder);
    if (compileMainPath) {
      args = await getMainCompilationFile();
    }
    else {
      args = lastActiveEditor.document.uri.fsPath
    }
  }

  workspaceFolder = workspace.getWorkspaceFolder(URI.file(args));
  const serverOptions: ServerOptions = getConfig(Section.SourcePawn, "SourceServerOptions", workspaceFolder);

  if (serverOptions === undefined) {
    window.showInformationMessage(
      "No server details were defined.",
      "Open Settings"
    )
      .then((choice) => {
        if (choice === "Open Settings") {
          commands.executeCommand(
            "workbench.action.openSettings",
            "@ext:sarrus.sourcepawn-vscode"
          );
        }
      });
    return 1;
  }

  // Return if no server commands were defined to run
  const serverCommands: string[] = getConfig(Section.SourcePawn, "serverCommands", workspaceFolder);
  if (serverCommands.length == 0) {
    window.showInformationMessage(
      "No commands have been specified to run.",
      "Open Settings"
    )
      .then((choice) => {
        if (choice === "Open Settings") {
          commands.executeCommand(
            "workbench.action.openSettings",
            "@ext:sarrus.sourcepawn-vscode"
          );
        }
      });
    return 1;
  }

  // Return if the server options were not properly defined
  if (serverOptions["host"] == "" || serverOptions["password"] == "") {
    window.showErrorMessage(
      "The host or the password were not set.",
      "Open Settings"
    )
      .then((choice) => {
        if (choice === "Open Settings") {
          commands.executeCommand(
            "workbench.action.openSettings",
            "@ext:sarrus.sourcepawn-vscode"
          );
        }
      });
    return 1;
  }

  // Setup RCON details
  const server = new Rcon({
    host: serverOptions["host"],
    port: serverOptions["port"],
    encoding: serverOptions["encoding"],
    timeout: serverOptions["timeout"],
  });

  // Fire the commands
  try {
    await server.authenticate(serverOptions["password"]);
    serverCommands.forEach(async (command) => {
      command = command.replace('${plugin}', getPluginName(args));
      const runCommands = await server.execute(command);
      console.log(runCommands);
    });
    return 0;
  } catch (e) {
    // TODO: inform the user
    console.error(e);
    return 2;
  }
}
