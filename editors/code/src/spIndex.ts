import * as vscode from "vscode";
import { URI } from "vscode-uri";
import * as lc from "vscode-languageclient/node";

import { createServerCommands, registerSMCommands } from "./Commands/registerCommands";
import { SMDocumentFormattingEditProvider } from "./Formatters/spFormat";
import { KVDocumentFormattingEditProvider } from "./Formatters/kvFormat";

import { Ctx } from "./ctx";
import { registerKVLinter } from "./Keyvalues/registerKVLinter";
import { buildDoctorStatusBar } from "./Commands/doctor";
import path from "path";
import { Section, getConfig } from "./configUtils";

export let defaultContext: Ctx;
export const serverContexts: Map<string, Ctx> = new Map();
export let lastActiveEditor: vscode.TextEditor;

export async function activate(context: vscode.ExtensionContext) {
  function didOpenTextDocument(document: vscode.TextDocument): void {
    // We are only interested in sourcepawn files.
    if (document.languageId !== "sourcepawn" || document.uri.scheme !== "file") {
      return;
    }

    const uri = document.uri;
    // Untitled files go to a default client.
    if (uri.scheme === "untitled" && !defaultContext) {
      const clientOptions: lc.LanguageClientOptions = {
        documentSelector: [{ scheme: "untitled", language: "sourcepawn" }],
      };
      defaultContext = new Ctx("default", context, createServerCommands(), clientOptions);
      defaultContext.start();
      return;
    }
    let folder = vscode.workspace.getWorkspaceFolder(uri);
    let parentDirectory = path.dirname(uri.fsPath);
    let parentDirectoryUri = URI.file(parentDirectory);
    if (folder) {
      // If we have nested workspace folders we only start a server on the outer most workspace folder.
      folder = getOuterMostWorkspaceFolder(folder);
      parentDirectory = folder.uri.fsPath;
      parentDirectoryUri = URI.file(parentDirectory);
    } else {
      folder = { uri: parentDirectoryUri, name: parentDirectory, index: -1 };
    }

    if (serverContexts.has(parentDirectoryUri.toString())) {
      return;
    }

    // TODO: Check if we should update the pattern here when the options change.
    const documentSelector: lc.DocumentSelector = [
      {
        scheme: "file",
        language: "sourcepawn",
        pattern: `${parentDirectory}/**/*.{inc,sp}`,
      },
    ].concat(
      getConfig(Section.LSP, "includeDirectories", undefined, []).map((e) => {
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
        fileEvents: vscode.workspace.createFileSystemWatcher(`${parentDirectory}/**/*.{inc,sp}`),
      },
    };
    let ctx = new Ctx(parentDirectoryUri.toString(), context, createServerCommands(), clientOptions);
    ctx.start();
    serverContexts.set(parentDirectoryUri.toString(), ctx);
  }

  migrateSettings();

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
      if (editor.document.uri.scheme === "output") {
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

  context.subscriptions.push(vscode.workspace.onDidChangeWorkspaceFolders(() => (_sortedWorkspaceFolders = undefined)));

  // Register KV linter
  registerKVLinter(context);

  // Set the last opened tab as the active document
  vscode.window.visibleTextEditors.forEach((editor) => {
    if (path.isAbsolute(editor.document.fileName)) {
      lastActiveEditor = editor;
    }
  });
}

// TODO: Remove after migration is done
function migrateSettings() {
  const oldIncludeDirs = vscode.workspace.getConfiguration(Section.LSP).get<string[]>("includesDirectories", []);
  const newIncludeDirs = vscode.workspace.getConfiguration(Section.LSP).get<string[]>("includeDirectories", []);
  if (newIncludeDirs.length === 0) {
    vscode.workspace.getConfiguration(Section.LSP).update("includeDirectories", oldIncludeDirs, true);
  }

  const oldPath = vscode.workspace.getConfiguration(Section.LSP).get<string>("spcompPath", "");
  const newPath = vscode.workspace.getConfiguration(Section.LSP).get<string>("compiler.path", "");
  if (newPath === null) {
    vscode.workspace.getConfiguration(Section.LSP).update("compiler.path", oldPath, true);
  }
}

export function getCtxFromUri(uri: URI): Ctx | undefined {
  let folder = vscode.workspace.getWorkspaceFolder(uri);
  if (!folder) {
    return defaultContext;
  }
  // If we have nested workspace folders we only start a server on the outer most workspace folder.
  folder = getOuterMostWorkspaceFolder(folder);
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

export function getOuterMostWorkspaceFolder(folder: vscode.WorkspaceFolder): vscode.WorkspaceFolder {
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
