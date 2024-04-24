import * as vscode from "vscode";
import path from "path";

export function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export function isSPFile(fileName: string) {
  return /(?:\.sp|\.inc)\s*^/.test(fileName);
}

export function getPluginName(uri: string): string {
  const fileName = path.basename(uri);
  const pluginName = fileName.split('.')[0];
  return pluginName;
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
