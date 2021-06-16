import vscode = require("vscode");
import Rcon from "rcon-srcds";

export async function run(args: any) {
  const serverOptions: Object = vscode.workspace
    .getConfiguration("sourcepawn")
    .get("SourceServerOptions");
  if (serverOptions["host"] == "" || serverOptions["password"] == "") {
    vscode.window
      .showErrorMessage(
        "The host or the password was not set.",
        "Open Settings"
      )
      .then((choice) => {
        if (choice === "Open Settings") {
          vscode.commands.executeCommand(
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
