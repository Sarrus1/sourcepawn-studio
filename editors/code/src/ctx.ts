import { platform } from "os";
import { join, resolve } from "path";
import * as vscode from "vscode";
import * as lc from "vscode-languageclient/node";

import * as lsp_ext from "./lsp_ext";
import { LazyOutputChannel } from "./spUtils";
import { createClient } from "./client";
import { execFile } from "child_process";
import { Section, getConfig } from "./configUtils";

export type CommandFactory = {
  enabled: (ctx: CtxInit) => Cmd;
  disabled?: (ctx: Ctx) => Cmd;
};

export type CtxInit = Ctx & {
  readonly client: lc.LanguageClient;
};

export class Ctx {
  readonly serverStatusBar: vscode.StatusBarItem;
  readonly spcompStatusBar: vscode.StatusBarItem;

  private _client: lc.LanguageClient | undefined;
  private _serverPath: string | undefined;
  private traceOutputChannel: vscode.OutputChannel | undefined;
  private outputChannel: vscode.OutputChannel | undefined;
  private clientSubscriptions: Disposable[];
  private commandFactories: Record<string, CommandFactory>;
  private commandDisposables: Disposable[];
  private clientOptions: lc.LanguageClientOptions;

  constructor(
    readonly folder: string,
    readonly extCtx: vscode.ExtensionContext,
    commandFactories: Record<string, CommandFactory>,
    clientOptions: lc.LanguageClientOptions
  ) {
    extCtx.subscriptions.push(this);
    this._serverPath = join(
      vscode.extensions.getExtension("Sarrus.sourcepawn-vscode").extensionPath,
      "languageServer",
      platform() == "win32" ? "sourcepawn-studio.exe" : "sourcepawn-studio"
    );
    this.clientOptions = clientOptions;

    this.serverStatusBar = vscode.window.createStatusBarItem(
      vscode.StatusBarAlignment.Left
    );
    this.serverStatusBar.show();

    this.spcompStatusBar = vscode.window.createStatusBarItem(
      vscode.StatusBarAlignment.Left
    );
    this.spcompStatusBar.show();

    this.clientSubscriptions = [];
    this.commandDisposables = [];
    this.commandFactories = commandFactories;
    try {
      this.updateCommands("disable");
    } catch {
      console.error(
        "Commands are already registered and will be handled later..."
      );
    }
    this.setServerStatus({
      health: "stopped",
    });
    this.setSpcompStatus({
      quiescent: true,
    });
  }

  public hideServer() {
    this.serverStatusBar.hide();
    this.spcompStatusBar.hide();
    this.commandDisposables.forEach((disposable) => disposable.dispose());
    this.commandDisposables = [];
  }

  public showServer() {
    this.serverStatusBar.show();
    this.spcompStatusBar.show();
    this.updateCommands();
  }

  dispose() {
    this.serverStatusBar.dispose();
    void this.disposeClient();
    this.commandDisposables.forEach((disposable) => disposable.dispose());
  }

  async getServerVersionFromBinaryAsync(): Promise<string | undefined> {
    const childProcess = await execFile(this._serverPath, ["--version"]);
    if (childProcess.stderr) {
      return undefined;
    }
    return childProcess.stdout
      .toString()
      .trim()
      .match(/^sourcepawn-studio (\d+\.\d+\.\d+)$/)[1];
  }

  private async getOrCreateClient() {
    if (!this.traceOutputChannel) {
      this.traceOutputChannel = new LazyOutputChannel(
        "SourcePawn Language Server Trace"
      );
      this.pushExtCleanup(this.traceOutputChannel);
    }
    if (!this.outputChannel) {
      this.outputChannel = vscode.window.createOutputChannel(
        "SourcePawn Language Server"
      );
      this.pushExtCleanup(this.outputChannel);
    }

    if (!this._client || !this._client.isRunning()) {
      const traceServer = getConfig(Section.SourcePawn, "trace.server");
      let traceServerLevel = 0;
      switch (traceServer) {
        case "warn":
          traceServerLevel = 1;
          break;
        case "info":
          traceServerLevel = 2;
          break;
        case "debug":
          traceServerLevel = 3;
          break;
        case "trace":
          traceServerLevel = 4;
          break;
      }
      let args = [];
      if (traceServerLevel > 0) {
        args.push(`-${"v".repeat(traceServerLevel)}`);
      }
      const serverOptions: lc.ServerOptions = {
        run: {
          command: this._serverPath,
          args,
        },
        debug: {
          command: resolve(
            process.env["__SOURCEPAWN_LSP_SERVER_DEBUG"] +
              (platform() == "win32" ? ".exe" : "")
          ),
          args: ["-vvv"],
        },
      };

      this._client = await createClient(
        this.traceOutputChannel,
        this.outputChannel,
        serverOptions,
        this.clientOptions
      );

      this.pushClientCleanup(
        this._client.onNotification(lsp_ext.serverStatus, (params) =>
          this.setServerStatus(params)
        )
      );
      this.pushClientCleanup(
        this._client.onNotification(lsp_ext.spcompStatus, (params) =>
          this.setSpcompStatus(params)
        )
      );
      // this.pushClientCleanup(
      //   this._client.onNotification(lsp_ext.openServerLogs, () => {
      //     this.outputChannel!.show();
      //   })
      // );
      this.pushClientCleanup(
        vscode.workspace.onDidChangeConfiguration((event) => {
          if (event.affectsConfiguration("SourcePawnLanguageServer")) {
            this._client.sendNotification(
              lc.DidChangeConfigurationNotification.type,
              {
                settings: {},
              }
            );
          }
        })
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
    this.updateCommands();
  }

  async restart() {
    await this.stopAndDispose();
    await this.start();
  }

  async stop() {
    if (!this._client) {
      return;
    }
    this.updateCommands("disable");
    // Increase the timeout in-case the server is parsing a file.
    await this._client.stop(10 * 1000);
  }

  async stopAndDispose() {
    if (!this._client) {
      return;
    }
    this.updateCommands("disable");
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
      const fullName = `sourcepawn-vscode.${name}`;
      let callback;
      if (isClientRunning(this)) {
        // we asserted that `client` is defined
        callback = factory.enabled(this);
      } else if (factory.disabled) {
        callback = factory.disabled(this);
      } else {
        callback = () =>
          vscode.window.showErrorMessage(
            `command ${fullName} failed: sourcepawn-studio is not running`
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
    const statusBar = this.serverStatusBar;
    statusBar.tooltip = new vscode.MarkdownString("", true);
    statusBar.tooltip.isTrusted = true;
    switch (status.health) {
      case "ok":
        statusBar.tooltip.appendText(status.message ?? "Ready");
        statusBar.command = "sourcepawn-vscode.stopServer";
        statusBar.color = undefined;
        statusBar.backgroundColor = undefined;
        break;
      case "warning":
        if (status.message) {
          statusBar.tooltip.appendText(status.message);
        }
        statusBar.command = "sourcepawn-vscode.stopServer";
        statusBar.color = new vscode.ThemeColor(
          "statusBarItem.warningForeground"
        );
        statusBar.backgroundColor = new vscode.ThemeColor(
          "statusBarItem.warningBackground"
        );
        icon = "$(warning) ";
        break;
      case "error":
        if (status.message) {
          statusBar.tooltip.appendText(status.message);
        }

        statusBar.command = "sourcepawn-vscode.stopServer";
        statusBar.color = new vscode.ThemeColor(
          "statusBarItem.errorForeground"
        );
        statusBar.backgroundColor = new vscode.ThemeColor(
          "statusBarItem.errorBackground"
        );
        icon = "$(error) ";
        break;
      case "stopped":
        statusBar.tooltip.appendText("Server is stopped");
        statusBar.tooltip.appendMarkdown(
          "\n\n[Start server](command:sourcepawn-vscode.startServer)"
        );
        statusBar.command = "sourcepawn-vscode.startServer";
        statusBar.color = undefined;
        statusBar.backgroundColor = undefined;
        statusBar.text = `$(stop-circle) sourcepawn-studio`;
        this.setSpcompStatus({
          quiescent: true,
        });
        return;
    }
    if (statusBar.tooltip.value) {
      statusBar.tooltip.appendText("\n\n");
    }
    statusBar.tooltip.appendMarkdown(
      "\n\n[Open logs](command:sourcepawn-vscode.openLogs)"
    );
    // TODO:
    // statusBar.tooltip.appendMarkdown(
    //   "\n\n[Reload Workspace](command:sourcepawn-vscode.reloadWorkspace)"
    // );
    statusBar.tooltip.appendMarkdown(
      "\n\n[Restart server](command:sourcepawn-vscode.startServer)"
    );
    statusBar.tooltip.appendMarkdown(
      "\n\n[Stop server](command:sourcepawn-vscode.stopServer)"
    );
    if (!status.quiescent) icon = "$(sync~spin) ";
    statusBar.text = `${icon}sourcepawn-studio`;
  }

  setSpcompStatus(status: lsp_ext.SpcompStatusParams) {
    const statusBar = this.spcompStatusBar;
    if (status.quiescent) {
      statusBar.hide();
    } else {
      statusBar.show();
      statusBar.tooltip = "spcomp is running";
      statusBar.color = undefined;
      statusBar.backgroundColor = undefined;
      statusBar.text = `$(sync~spin) spcomp`;
    }
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
