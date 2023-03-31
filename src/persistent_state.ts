import * as vscode from "vscode";

export class PersistentState {
  constructor(private readonly globalState: vscode.Memento) {}

  /**
   * Version of the extension that installed the server.
   */
  get serverVersion(): string | undefined {
    return this.globalState.get("serverVersion");
  }
  async updateServerVersion(value: string | undefined) {
    await this.globalState.update("serverVersion", value);
  }
}
