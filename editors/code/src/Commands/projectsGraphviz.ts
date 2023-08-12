import * as vscode from "vscode";
import { projectsGraphviz, ProjectsGraphvizParams } from "../lsp_ext";
import { sleep } from "../spUtils";
import { Cmd, CtxInit } from "../ctx";

export function projectsGraphvizCommand(ctx: CtxInit): Cmd {
  const tdcp = new (class implements vscode.TextDocumentContentProvider {
    readonly uri = vscode.Uri.parse(
      "sourcepawn-lsp-projects-graphviz-file://graphvizFile/projects.gv"
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
      console.log("Received request.");
      const params: ProjectsGraphvizParams = {};
      const doc = vscode.window.activeTextEditor?.document;
      console.log("Received request1.");
      if (doc !== undefined) {
        params.textDocument =
          ctx?.client.code2ProtocolConverter.asTextDocumentIdentifier(doc);
      }
      console.log("Received request2.");
      console.log(params);
      const text = await ctx?.client.sendRequest(projectsGraphviz, params);
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
      "sourcepawn-lsp-projects-graphviz-file",
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

function isSPFile(fileName: string) {
  return /(?:\.sp|\.inc)\s*^/.test(fileName);
}
