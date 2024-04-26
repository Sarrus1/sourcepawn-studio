import { workspace as workspace, window, commands } from "vscode";
import Rcon from "rcon-srcds";
import { EncodingOptions } from "rcon-srcds/dist/packet";
import { alwaysCompileMainPath, getMainCompilationFile, getPluginName } from "../spUtils";
import { lastActiveEditor } from "../spIndex";
import { URI } from "vscode-uri";

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
  // If we don't receive args, we need to figure out which plugin was sent
  if (!args) {
    if (await alwaysCompileMainPath()) {
      args = await getMainCompilationFile();
    }
    else {
      args = lastActiveEditor.document.uri.fsPath
    }
  }

  const workspaceFolder = workspace.getWorkspaceFolder(URI.file(args));

  // Return if no server options were configured
  const serverOptions: ServerOptions | undefined = workspace.getConfiguration(
    "sourcepawn",
    workspaceFolder
  ).get("SourceServerOptions");
  if (serverOptions === undefined) {
    window
      .showInformationMessage(
        "No server options to run the commands on were defined.",
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
  const serverCommands: string[] | undefined = workspace.getConfiguration(
    "sourcepawn",
    workspaceFolder
  ).get("serverCommands");
  if (serverCommands.length == 0) {
    window
      .showInformationMessage(
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
    window
      .showErrorMessage(
        "The host or the password was not set.",
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
