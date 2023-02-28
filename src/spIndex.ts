import { homedir } from "os";
import { join } from "path";
import {
  workspace as Workspace,
  ExtensionContext,
  languages,
  extensions,
} from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";
import { platform } from "os";
import { existsSync } from "fs";

import { registerSMCommands } from "./Commands/registerCommands";
import { SMDocumentFormattingEditProvider } from "./Formatters/spFormat";
import { KVDocumentFormattingEditProvider } from "./Formatters/kvFormat";
import {
  getLatestVersionName,
  run as installLanguageServerCommand,
} from "./Commands/installLanguageServer";
import { migrateSettings } from "./spUtils";

export let client: LanguageClient;

function makeCommand() {
  return join(
    extensions.getExtension("Sarrus.sourcepawn-vscode").extensionPath,
    "languageServer",
    platform() == "win32" ? "sourcepawn_lsp.exe" : "sourcepawn_lsp"
  );
}

async function installLanguageServer(context: ExtensionContext) {
  const lspPath = join(
    extensions.getExtension("Sarrus.sourcepawn-vscode").extensionPath,
    "languageServer"
  );
  if (!existsSync(lspPath)) {
    await installLanguageServerCommand(undefined);
    const version = await getLatestVersionName();
    context.globalState.update("language_server_version", version);
  }
}

async function checkForLanguageServerUpdate(context: ExtensionContext) {
  const latestVersion = await getLatestVersionName();
  const installedVersion = context.globalState.get("language_server_version");
  if (
    latestVersion === undefined ||
    installedVersion === undefined ||
    latestVersion === installedVersion
  ) {
    return;
  }
  await client.stop();
  await installLanguageServerCommand(undefined);
  context.globalState.update("language_server_version", latestVersion);
  client.start();
}

export async function activate(context: ExtensionContext) {
  await installLanguageServer(context);

  registerSMCommands(context);

  context.subscriptions.push(
    languages.registerDocumentFormattingEditProvider(
      {
        language: "sourcepawn",
        scheme: "file",
      },
      new SMDocumentFormattingEditProvider()
    )
  );

  context.subscriptions.push(
    languages.registerDocumentFormattingEditProvider(
      {
        language: "valve-kv",
      },
      new KVDocumentFormattingEditProvider()
    )
  );

  const serverOptions: ServerOptions = {
    run: {
      command: makeCommand(),
      args: [],
    },
    debug: {
      command: "cargo",
      args: [
        "run",
        "--manifest-path",
        join(homedir(), "dev/sourcepawn-lsp/Cargo.toml"),
      ],
    },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ language: "sourcepawn" }],
    synchronize: {
      fileEvents: Workspace.createFileSystemWatcher("**/*.{inc,sp}"),
    },
  };

  client = new LanguageClient(
    "SourcePawnLanguageServer",
    serverOptions,
    clientOptions
  );

  client.start();
  try {
    checkForLanguageServerUpdate(context);
  } catch (error) {
    console.error("Couldn't update the language server.", error);
  }

  migrateSettings(client);
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
