import { homedir } from "os";
import { join } from "path";
import { workspace, ExtensionContext } from "vscode";

import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient;

export function activate(context: ExtensionContext) {
  const command = "cargo";
  const manifestPath = join(homedir(), "dev/sourcepawn-lsp/Cargo.toml");
  const args = ["run", "--manifest-path", manifestPath];
  const serverOptions: ServerOptions = {
    run: {
      command,
      args,
    },
    debug: {
      command,
      args,
    },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ language: "sourcepawn" }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher("**/*.{inc,sp}"),
    },
  };

  client = new LanguageClient(
    "SourcePawnLanguageServer",
    serverOptions,
    clientOptions
  );

  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
