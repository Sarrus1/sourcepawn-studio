import { workspace as workspace, window, commands, WorkspaceFolder, ProgressLocation } from "vscode";
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
  if (serverOptions.host == "") {
    window.showErrorMessage(
      "The host was not set.",
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

  // Begin progress
  window.withProgress(
    {
      title: "Executing commands...",
      location: ProgressLocation.Notification,
      cancellable: false,
    },
    async () => {
      try {
        // Attempt to connect
        await server.authenticate(serverOptions["password"]);

        // Run commands
        for (const command of serverCommands) {
          const modifiedCommand = command.replace('${plugin}', getPluginName(args));
          server.execute(modifiedCommand);
        }

        window.showInformationMessage("Commands executed successfully!");
        return 0;
      } catch (error) {
        window.showErrorMessage(`Failed to run commands! ${error}`);
        return 1;
      }
    }
  );
}
