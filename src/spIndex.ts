import * as vscode from "vscode";

import {
  createServerCommands,
  registerSMCommands,
} from "./Commands/registerCommands";
import { SMDocumentFormattingEditProvider } from "./Formatters/spFormat";
import { KVDocumentFormattingEditProvider } from "./Formatters/kvFormat";
import {
  getLatestVersionName,
  run as installLanguageServerCommand,
} from "./Commands/installLanguageServer";
import { migrateSettings } from "./spUtils";
import { Ctx } from "./ctx";

export let ctx: Ctx | undefined;

async function checkForLanguageServerUpdate(context: vscode.ExtensionContext) {
  if (context.extensionMode === vscode.ExtensionMode.Development) {
    return;
  }
  const latestVersion = await getLatestVersionName();
  const installedVersion = context.globalState.get("language_server_version");
  if (
    latestVersion === undefined ||
    installedVersion === undefined ||
    latestVersion === installedVersion
  ) {
    return;
  }
  await installLanguageServerCommand(undefined);
  context.globalState.update("language_server_version", latestVersion);
}

export async function activate(context: vscode.ExtensionContext) {
  migrateSettings(ctx);
  ctx = new Ctx(context, createServerCommands());
  ctx.start();

  registerSMCommands(context);

  context.subscriptions.push(
    vscode.languages.registerDocumentFormattingEditProvider(
      {
        language: "sourcepawn",
        scheme: "file",
      },
      new SMDocumentFormattingEditProvider()
    )
  );

  context.subscriptions.push(
    vscode.languages.registerDocumentFormattingEditProvider(
      {
        language: "valve-kv",
      },
      new KVDocumentFormattingEditProvider()
    )
  );

  try {
    checkForLanguageServerUpdate(context);
  } catch (error) {
    console.error("Couldn't update the language server.", error);
  }
}
