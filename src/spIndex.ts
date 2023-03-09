import * as vscode from "vscode";

import {
  createServerCommands,
  registerSMCommands,
} from "./Commands/registerCommands";
import { SMDocumentFormattingEditProvider } from "./Formatters/spFormat";
import { KVDocumentFormattingEditProvider } from "./Formatters/kvFormat";

import { migrateSettings } from "./spUtils";
import { Ctx } from "./ctx";

export let ctx: Ctx | undefined;

export async function activate(context: vscode.ExtensionContext) {
  migrateSettings();
  ctx = new Ctx(context, createServerCommands());
  ctx.start().then(() => {
    try {
      ctx.checkForLanguageServerUpdate();
    } catch (error) {
      console.error("Couldn't update the language server.", error);
    }
  });

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
}
