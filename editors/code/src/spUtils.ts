import * as vscode from "vscode";
import path from "path";
import { getCtxFromUri, lastActiveEditor } from "./spIndex";
import { ProjectMainPathParams, projectMainPath } from "./lsp_ext";
import { URI } from "vscode-uri";

export function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export function isSPFile(filePath: string): boolean {
  return /\.(sp|inc)$/i.test(filePath);
}

export function getPluginName(uri: string): string {
  const fileName = path.basename(uri);
  const pluginName = fileName.split('.')[0];
  return pluginName;
}

export async function getMainCompilationFile(): Promise<string> {
  const uri = lastActiveEditor.document.uri;
  const params: ProjectMainPathParams = {
    uri: uri.toString(),
  };
  const mainUri = await getCtxFromUri(uri)?.client.sendRequest(
    projectMainPath,
    params
  );
  return URI.parse(mainUri).fsPath;
}

export async function alwaysCompileMainPath(): Promise<boolean> {
  const workspaceFolder =
    vscode.workspace.getWorkspaceFolder(lastActiveEditor.document.uri)
  return vscode.workspace.getConfiguration(
    "sourcepawn",
    workspaceFolder
  ).get<boolean>("MainPathCompilation");
}

export class LazyOutputChannel implements vscode.OutputChannel {
  constructor(name: string) {
    this.name = name;
  }

  name: string;
  _channel: vscode.OutputChannel | undefined;

  get channel(): vscode.OutputChannel {
    if (!this._channel) {
      this._channel = vscode.window.createOutputChannel(this.name);
    }
    return this._channel;
  }

  append(value: string): void {
    this.channel.append(value);
  }
  appendLine(value: string): void {
    this.channel.appendLine(value);
  }
  replace(value: string): void {
    this.channel.replace(value);
  }
  clear(): void {
    if (this._channel) {
      this._channel.clear();
    }
  }
  show(preserveFocus?: boolean): void;
  show(column?: vscode.ViewColumn, preserveFocus?: boolean): void;
  show(column?: any, preserveFocus?: any): void {
    this.channel.show(column, preserveFocus);
  }
  hide(): void {
    if (this._channel) {
      this._channel.hide();
    }
  }
  dispose(): void {
    if (this._channel) {
      this._channel.dispose();
    }
  }
}
