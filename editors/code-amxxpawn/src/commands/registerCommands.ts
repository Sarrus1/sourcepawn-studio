import * as vscode from "vscode";

import { run as setFileAsMainCommand } from "./setFileAsMain";
import { CommandFactory } from "../ctx";

/**
 * Register all the vscode.commands of the extension.
 * @param  {vscode.ExtensionContext} context The extension's context.
 * @returns void
 */
export function registerSMCommands(context: vscode.ExtensionContext): void {
  const setFileAsMain = vscode.commands.registerCommand(
    "amxxpawn-vscode.installLanguageServer",
    setFileAsMainCommand.bind(undefined)
  );
  context.subscriptions.push(setFileAsMain);
}

/**
 * Prepare a record of server specific commands.
 * @returns Record
 */
export function createServerCommands(): Record<string, CommandFactory> {
  return {
    reload: {
      enabled: (ctx) => async () => {
        void vscode.window.showInformationMessage(
          "Reloading sourcepawn-studio..."
        );
        await ctx.restart();
      },
      disabled: (ctx) => async () => {
        void vscode.window.showInformationMessage(
          "Reloading sourcepawn-studio..."
        );
        await ctx.start();
      },
    },
    startServer: {
      enabled: (ctx) => async () => {
        await ctx.start();
      },
      disabled: (ctx) => async () => {
        await ctx.start();
      },
    },
    stopServer: {
      enabled: (ctx) => async () => {
        await ctx.stopAndDispose();
        ctx.setServerStatus({
          health: "stopped",
        });
      },
      disabled: (_) => async () => {},
    },
  };
}
