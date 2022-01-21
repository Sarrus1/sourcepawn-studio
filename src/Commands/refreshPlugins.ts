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
  let workspaceFolder =
    args === undefined ? undefined : Workspace.getWorkspaceFolder(args);
  const serverOptions: ServerOptions | undefined = Workspace.getConfiguration(
    "sourcepawn",
    workspaceFolder
  ).get("SourceServerOptions");
  if (serverOptions === undefined) {
    return 0;
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
    let refresh = await server.execute("sm plugins refresh");
    console.log(refresh);
    return 0;
  } catch (e) {
    console.error(e);
    return 2;
  }
}
