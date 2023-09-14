import * as vscode from "vscode";

import {
  createServerCommands,
  registerSMCommands,
} from "./Commands/registerCommands";
import { SMDocumentFormattingEditProvider } from "./Formatters/spFormat";
import { KVDocumentFormattingEditProvider } from "./Formatters/kvFormat";

import { Ctx } from "./ctx";
import { registerKVLinter } from "./Keyvalues/registerKVLinter";
import { buildDoctorStatusBar } from "./Commands/doctor";

export let ctx: Ctx | undefined;

export async function activate(context: vscode.ExtensionContext) {
  ctx = new Ctx(context, createServerCommands());
  ctx.start();

  registerSMCommands(context);
  buildDoctorStatusBar();

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

  // Register KV linter
  registerKVLinter(context);
}
