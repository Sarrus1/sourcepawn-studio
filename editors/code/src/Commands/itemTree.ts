import * as vscode from "vscode";
import { itemTree, ItemTreeParams } from "../lsp_ext";
import { isSPFile, sleep } from "../spUtils";
import { Cmd, CtxInit } from "../ctx";

export function itemTreeCommand(ctx: CtxInit): Cmd {
  const tdcp = new (class implements vscode.TextDocumentContentProvider {
    readonly uri = vscode.Uri.parse(
      "sourcepawn-studio-item-tree://itemTree/itemTree.sp"
    );
    readonly eventEmitter = new vscode.EventEmitter<vscode.Uri>();
    constructor() {
      vscode.workspace.onDidChangeTextDocument(
        this.onDidChangeTextDocument,
        this,
        ctx.subscriptions
      );
      vscode.window.onDidChangeActiveTextEditor(
        this.onDidChangeActiveTextEditor,
        this,
        ctx.subscriptions
      );
    }

    private onDidChangeTextDocument(event: vscode.TextDocumentChangeEvent) {
      if (isSPFile(event.document.fileName)) {
        // We need to order this after language server updates, but there's no API for that.
        // Hence, good old sleep().
        void sleep(10).then(() => this.eventEmitter.fire(this.uri));
      }
    }
    private onDidChangeActiveTextEditor(editor: vscode.TextEditor | undefined) {
      if (editor && isSPFile(editor.document.fileName)) {
        this.eventEmitter.fire(this.uri);
      }
    }

    async provideTextDocumentContent(
      _uri: vscode.Uri,
      ct: vscode.CancellationToken
    ): Promise<string> {
      const params: ItemTreeParams = {};
      const doc = vscode.window.activeTextEditor?.document;
      if (doc === undefined) {
        return "";
      }
      params.textDocument =
        ctx?.client.code2ProtocolConverter.asTextDocumentIdentifier(doc);
      if (params.textDocument === undefined) {
        return "";
      }
      const text = await ctx?.client.sendRequest(itemTree, params);
      if (text === undefined) {
        return "";
      }
      return text;
    }

    get onDidChange(): vscode.Event<vscode.Uri> {
      return this.eventEmitter.event;
    }
  })();

  ctx.pushExtCleanup(
    vscode.workspace.registerTextDocumentContentProvider(
      "sourcepawn-studio-item-tree",
      tdcp
    )
  );

  return async () => {
    const document = await vscode.workspace.openTextDocument(tdcp.uri);
    tdcp.eventEmitter.fire(tdcp.uri);
    void (await vscode.window.showTextDocument(document, {
      viewColumn: vscode.ViewColumn.Two,
      preserveFocus: true,
    }));
  };
}
