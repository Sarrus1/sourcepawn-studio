import * as vscode from "vscode";
import { URI } from "vscode-uri";
import * as lc from "vscode-languageclient/node";

import {
  createServerCommands,
  registerSMCommands,
} from "./Commands/registerCommands";
import { SMDocumentFormattingEditProvider } from "./Formatters/spFormat";
import { KVDocumentFormattingEditProvider } from "./Formatters/kvFormat";

import { Ctx } from "./ctx";
import { registerKVLinter } from "./Keyvalues/registerKVLinter";
import { buildDoctorStatusBar } from "./Commands/doctor";
import path from "path";
import { Section, getConfig } from "./configUtils";

export let defaultContext: Ctx;
export const serverContexts: Map<string, Ctx> = new Map();
export let lastActiveEditor: vscode.TextEditor

export async function activate(context: vscode.ExtensionContext) {
  function didOpenTextDocument(document: vscode.TextDocument): void {
    // We are only interested in sourcepawn files.
    if (
      document.languageId !== "sourcepawn" ||
      (document.uri.scheme !== "file" && document.uri.scheme !== "untitled")
    ) {
      return;
    }

    const uri = document.uri;
    // Untitled files go to a default client.
    if (uri.scheme === "untitled" && !defaultContext) {
      const clientOptions: lc.LanguageClientOptions = {
        documentSelector: [{ scheme: "untitled", language: "sourcepawn" }],
      };
      defaultContext = new Ctx(
        "default",
        context,
        createServerCommands(),
        clientOptions
      );
      defaultContext.start();
      return;
    }
    let folder = vscode.workspace.getWorkspaceFolder(uri);
    if (!folder) {
      return;
    }
    // If we have nested workspace folders we only start a server on the outer most workspace folder.
    folder = getOuterMostWorkspaceFolder(folder);
    if (!serverContexts.has(folder.uri.toString())) {
      // TODO: Check if we should update the pattern here when the options change.
      const documentSelector: lc.DocumentSelector = [
        {
          scheme: "file",
          language: "sourcepawn",
          pattern: `${folder.uri.fsPath}/**/*.{inc,sp}`,
        },
      ].concat(
        getConfig(Section.LSP, "includeDirectories", undefined, [])
          .map((e) => {
            return {
              scheme: "file",
              language: "sourcepawn",
              pattern: `${e}/**/*.{inc,sp}`,
            };
          })
      );
      const clientOptions: lc.LanguageClientOptions = {
        documentSelector,
        workspaceFolder: folder,
        synchronize: {
          fileEvents: vscode.workspace.createFileSystemWatcher(
            `${folder.uri.fsPath}/**/*.{inc,sp}`
          ),
        },
      };
      let ctx = new Ctx(
        folder.uri.toString(),
        context,
        createServerCommands(),
        clientOptions
      );
      ctx.start();
      serverContexts.set(folder.uri.toString(), ctx);
    }
  }

  vscode.workspace.onDidOpenTextDocument(didOpenTextDocument);
  vscode.workspace.textDocuments.forEach(didOpenTextDocument);
  vscode.workspace.onDidChangeWorkspaceFolders((event) => {
    for (const folder of event.removed) {
      const ctx = serverContexts.get(folder.uri.toString());
      if (ctx) {
        serverContexts.delete(folder.uri.toString());
        ctx.stop();
      }
    }
  });

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

  context.subscriptions.push(
    vscode.window.onDidChangeActiveTextEditor((editor) => {
      // We check for a valid path in the editor's filename,
      // which would indicate we're not on an output console
      if (!path.isAbsolute(editor.document.fileName)) {
        return;
      }

      // We save the last editor that's not an output console
      if (lastActiveEditor != editor) {
        lastActiveEditor = editor;
      }

      let folder = vscode.workspace.getWorkspaceFolder(editor.document.uri);
      folder = getOuterMostWorkspaceFolder(folder);
      serverContexts.forEach((ctx, _) => ctx.hideServer());
      const ctx = serverContexts.get(folder.uri.toString());
      if (ctx === undefined) {
        defaultContext.showServer();
        return;
      }
      serverContexts.get(folder.uri.toString())?.showServer();
    })
  );

  context.subscriptions.push(
    vscode.workspace.onDidChangeWorkspaceFolders(
      () => (_sortedWorkspaceFolders = undefined)
    )
  );

  // Register KV linter
  registerKVLinter(context);

  // Set the last opened tab as the active document
  vscode.window.visibleTextEditors.forEach(editor => {
    if (path.isAbsolute(editor.document.fileName)) {
      lastActiveEditor = editor;
    }
  })
}

export function getCtxFromUri(uri: URI): Ctx | undefined {
  let folder = vscode.workspace.getWorkspaceFolder(uri);
  if (!folder) {
    return defaultContext;
  }
  // If we have nested workspace folders we only start a server on the outer most workspace folder.
  folder = getOuterMostWorkspaceFolder(folder);
  console.log(serverContexts);
  return serverContexts.get(folder.uri.toString());
}

let _sortedWorkspaceFolders: string[] | undefined;
function sortedWorkspaceFolders(): string[] {
  if (_sortedWorkspaceFolders === void 0) {
    _sortedWorkspaceFolders = vscode.workspace.workspaceFolders
      ? vscode.workspace.workspaceFolders
        .map((folder) => {
          let result = folder.uri.toString();
          if (result.charAt(result.length - 1) !== "/") {
            result = result + "/";
          }
          return result;
        })
        .sort((a, b) => {
          return a.length - b.length;
        })
      : [];
  }
  return _sortedWorkspaceFolders;
}

export function getOuterMostWorkspaceFolder(
  folder: vscode.WorkspaceFolder
): vscode.WorkspaceFolder {
  const sorted = sortedWorkspaceFolders();
  for (const element of sorted) {
    let uri = folder.uri.toString();
    if (uri.charAt(uri.length - 1) !== "/") {
      uri = uri + "/";
    }
    if (uri.startsWith(element)) {
      return vscode.workspace.getWorkspaceFolder(URI.parse(element))!;
    }
  }
  return folder;
}
