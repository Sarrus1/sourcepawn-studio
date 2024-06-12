import * as vscode from "vscode";
import { analyzerStatus, AnalyzerStatusParams } from "../lsp_ext";
import { Cmd, CtxInit } from "../ctx";

export function analyzerStatusCommand(ctx: CtxInit): Cmd {
  const tdcp = new (class implements vscode.TextDocumentContentProvider {
    readonly uri = vscode.Uri.parse("sourcepawn-studio-status://status");
    readonly eventEmitter = new vscode.EventEmitter<vscode.Uri>();

    async provideTextDocumentContent(
      _uri: vscode.Uri,
      ct: vscode.CancellationToken
    ): Promise<string> {
      const params: AnalyzerStatusParams = {};
      const doc = vscode.window.activeTextEditor?.document;
      if (doc === undefined) {
        return "";
      }
      params.textDocument =
        ctx?.client.code2ProtocolConverter.asTextDocumentIdentifier(doc);
      if (params.textDocument === undefined) {
        return "";
      }
      const text = await ctx?.client.sendRequest(analyzerStatus, params);
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
      "sourcepawn-studio-status",
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
