import { existsSync } from "fs";
import { homedir, platform } from "os";
import { join } from "path";
import * as vscode from "vscode";
import * as lc from "vscode-languageclient/node";

import * as lsp_ext from "./lsp_ext";
import { PersistentState } from "./persistent_state";
import {
  getLatestVersionName,
  run as installLanguageServerCommand,
} from "./Commands/installLanguageServer";

export type CommandFactory = {
  enabled: (ctx: CtxInit) => Cmd;
  disabled?: (ctx: Ctx) => Cmd;
};

export type CtxInit = Ctx & {
  readonly client: lc.LanguageClient;
};

export class Ctx {
  readonly statusBar: vscode.StatusBarItem;

  private _client: lc.LanguageClient | undefined;
  private _serverPath: string | undefined;
  private clientSubscriptions: Disposable[];
  private state: PersistentState;
  private commandFactories: Record<string, CommandFactory>;
  private commandDisposables: Disposable[];

  constructor(
    readonly extCtx: vscode.ExtensionContext,
    commandFactories: Record<string, CommandFactory>
  ) {
    extCtx.subscriptions.push(this);
    this._serverPath = join(
      vscode.extensions.getExtension("Sarrus.sourcepawn-vscode").extensionPath,
      "languageServer",
      platform() == "win32" ? "sourcepawn_lsp.exe" : "sourcepawn_lsp"
    );
    this.statusBar = vscode.window.createStatusBarItem(
      vscode.StatusBarAlignment.Left
    );
    this.statusBar.show();

    this.state = new PersistentState(extCtx.globalState);
    this.clientSubscriptions = [];
    this.commandDisposables = [];
    this.commandFactories = commandFactories;
    this.updateCommands("disable");
    this.setServerStatus({
      health: "stopped",
    });
  }

  dispose() {
    this.statusBar.dispose();
    void this.disposeClient();
    this.commandDisposables.forEach((disposable) => disposable.dispose());
  }

  private async installLanguageServerIfAbsent() {
    if (!existsSync(this._serverPath)) {
      await installLanguageServerCommand(undefined);
      const version = await getLatestVersionName();
      this.state.updateServerVersion(version);
    }
  }

  private async getOrCreateClient() {
    await this.installLanguageServerIfAbsent();
    if (!this._client) {
      const serverOptions: lc.ServerOptions = {
        run: {
          command: this._serverPath,
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

      const clientOptions: lc.LanguageClientOptions = {
        documentSelector: [{ language: "sourcepawn" }],
        synchronize: {
          fileEvents: vscode.workspace.createFileSystemWatcher("**/*.{inc,sp}"),
        },
      };

      this._client = new lc.LanguageClient(
        "SourcePawn Language Server",
        serverOptions,
        clientOptions
      );

      this.pushClientCleanup(
        this._client.onNotification(lsp_ext.serverStatus, (params) =>
          this.setServerStatus(params)
        )
      );
    }
    return this._client;
  }

  async start() {
    const client = await this.getOrCreateClient();
    if (!client) {
      return;
    }
    await client.start();
  }

  async restart() {
    await this.stopAndDispose();
    await this.start();
  }

  async stop() {
    if (!this._client) {
      return;
    }
    await this._client.stop();
  }

  async stopAndDispose() {
    if (!this._client) {
      return;
    }
    await this.disposeClient();
  }

  private async disposeClient() {
    this.clientSubscriptions?.forEach((disposable) => disposable.dispose());
    this.clientSubscriptions = [];
    await this._client?.dispose();
    this._client = undefined;
  }

  private updateCommands(forceDisable?: "disable") {
    this.commandDisposables.forEach((disposable) => disposable.dispose());
    this.commandDisposables = [];

    const clientRunning = (!forceDisable && this._client?.isRunning()) ?? false;
    const isClientRunning = function (_ctx: Ctx): _ctx is CtxInit {
      return clientRunning;
    };

    for (const [name, factory] of Object.entries(this.commandFactories)) {
      const fullName = `sourcepawn-lsp.${name}`;
      let callback;
      if (isClientRunning(this)) {
        // we asserted that `client` is defined
        callback = factory.enabled(this);
      } else if (factory.disabled) {
        callback = factory.disabled(this);
      } else {
        callback = () =>
          vscode.window.showErrorMessage(
            `command ${fullName} failed: sourcepawn_lsp is not running`
          );
      }

      this.commandDisposables.push(
        vscode.commands.registerCommand(fullName, callback)
      );
    }
  }

  get extensionPath(): string {
    return this.extCtx.extensionPath;
  }

  get subscriptions(): Disposable[] {
    return this.extCtx.subscriptions;
  }

  get serverPath(): string | undefined {
    return this._serverPath;
  }

  get client() {
    return this._client;
  }

  setServerStatus(status: lsp_ext.ServerStatusParams | { health: "stopped" }) {
    let icon = "";
    const statusBar = this.statusBar;
    switch (status.health) {
      case "ok":
        statusBar.tooltip =
          (status.message ?? "Ready") + "\nClick to stop server.";
        statusBar.command = "sourcepawn-lsp.stopServer";
        statusBar.color = undefined;
        statusBar.backgroundColor = undefined;
        break;
      case "warning":
        statusBar.tooltip =
          (status.message ? status.message + "\n" : "") + "Click to reload.";

        statusBar.command = "sourcepawn-lsp.reloadWorkspace";
        statusBar.color = new vscode.ThemeColor(
          "statusBarItem.warningForeground"
        );
        statusBar.backgroundColor = new vscode.ThemeColor(
          "statusBarItem.warningBackground"
        );
        icon = "$(warning) ";
        break;
      case "error":
        statusBar.tooltip =
          (status.message ? status.message + "\n" : "") + "Click to reload.";

        statusBar.command = "sourcepawn-lsp.reloadWorkspace";
        statusBar.color = new vscode.ThemeColor(
          "statusBarItem.errorForeground"
        );
        statusBar.backgroundColor = new vscode.ThemeColor(
          "statusBarItem.errorBackground"
        );
        icon = "$(error) ";
        break;
      case "stopped":
        statusBar.tooltip = "Server is stopped.\nClick to start.";
        statusBar.command = "sourcepawn-lsp.startServer";
        statusBar.color = undefined;
        statusBar.backgroundColor = undefined;
        statusBar.text = `$(stop-circle) sourcepawn-lsp`;
        return;
    }
    if (!status.quiescent) icon = "$(sync~spin) ";
    statusBar.text = `${icon}sourcepawn-lsp`;
  }

  pushExtCleanup(d: Disposable) {
    this.extCtx.subscriptions.push(d);
  }

  private pushClientCleanup(d: Disposable) {
    this.clientSubscriptions.push(d);
  }
}

export interface Disposable {
  dispose(): void;
}
export type Cmd = (...args: any[]) => unknown;
