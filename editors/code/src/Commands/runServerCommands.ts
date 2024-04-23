import { workspace as Workspace, window, commands } from "vscode";
import Rcon from "rcon-srcds";
import { EncodingOptions } from "rcon-srcds/dist/packet";

export interface ServerOptions {
  host: string;
  password: string;
  port: number;
  encoding: EncodingOptions;
  timeout: number;
}

export async function run(args: any) {
  const workspaceFolder =
    args === undefined ? undefined : Workspace.getWorkspaceFolder(args);
  const serverOptions: ServerOptions | undefined = Workspace.getConfiguration(
    "sourcepawn",
    workspaceFolder
  ).get("SourceServerOptions");
  const serverCommands: string[] | undefined = Workspace.getConfiguration(
    "sourcepawn",
    workspaceFolder
  ).get("serverCommands");
  if (serverOptions === undefined) {
    return 0;
  }
  if (serverCommands.length == 0) {
    window
      .showErrorMessage(
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
  const server = new Rcon({
    host: serverOptions["host"],
    port: serverOptions["port"],
    encoding: serverOptions["encoding"],
    timeout: serverOptions["timeout"],
  });
  try {
    await server.authenticate(serverOptions["password"]);
    serverCommands.forEach(async (command) => {
      const refresh = await server.execute(command);
      console.log(refresh);
    });
    return 0;
  } catch (e) {
    console.error(e);
    return 2;
  }
}
