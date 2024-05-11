import { workspace as Workspace, window, WorkspaceFolder } from "vscode";
import Rcon from "rcon-srcds";
import { EncodingOptions } from "rcon-srcds/dist/packet";
import { getMainCompilationFile, getPluginName } from "../spUtils";
import { lastActiveEditor } from "../spIndex";
import { URI } from "vscode-uri";
import { Section, editConfig, getConfig } from "../configUtils";

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
export async function run(args?: string): Promise<boolean> {
  let workspaceFolder: WorkspaceFolder;

  // If we don't receive args, we need to figure out which plugin was sent
  if (!args) {
    workspaceFolder = Workspace.getWorkspaceFolder(lastActiveEditor.document.uri);
    const compileMainPath: boolean = getConfig(Section.SourcePawn, "MainPathCompilation", workspaceFolder);
    if (compileMainPath) {
      args = await getMainCompilationFile();
    }
    else {
      args = lastActiveEditor.document.uri.fsPath
    }
  }

  workspaceFolder = Workspace.getWorkspaceFolder(URI.file(args));
  const serverOptions: ServerOptions = getConfig(Section.SourcePawn, "SourceServerOptions", workspaceFolder);
  if (!serverOptions || serverOptions.host === "" || serverOptions.port <= 0) {
    window.showInformationMessage(
      "No server details were defined, or host or port are empty/malformed.",
      "Open Settings"
    )
      .then((choice) => {
        if (choice === "Open Settings") {
          editConfig(Section.SourcePawn, "SourceServerOptions");
        }
      });
    return false;
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
          editConfig(Section.SourcePawn, "serverCommands");
        }
      });
    return false;
  }

  // Setup RCON details
  const server = new Rcon(serverOptions);

  try {
    // Attempt to connect
    await server.authenticate(serverOptions.password);

    // Run commands
    for (let command of serverCommands) {
      command = command.replace('${plugin}', getPluginName(args));
      server.execute(command);
    }

    window.showInformationMessage("Commands executed successfully!");
    return true;
  } catch (error) {
    window.showErrorMessage(`Failed to run commands! ${error}`);
    return false;
  }
  finally {
    await server.disconnect()
  }
}
