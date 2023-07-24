import * as vscode from "vscode";

import {
  createServerCommands,
  registerSMCommands,
} from "./commands/registerCommands";
import { SMDocumentFormattingEditProvider } from "./Formatters/spFormat";
import { KVDocumentFormattingEditProvider } from "./Formatters/kvFormat";

import { Ctx } from "./ctx";
import { registerKVLinter } from "./Keyvalues/registerKVLinter";

export let ctx: Ctx | undefined;

export async function activate(context: vscode.ExtensionContext) {
  ctx = new Ctx(context, createServerCommands());
  ctx.start();

  registerSMCommands(context);

  context.subscriptions.push(
    vscode.languages.registerDocumentFormattingEditProvider(
      {
        language: "amxxpawn",
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

  // Register KV linter
  registerKVLinter(context);
}
