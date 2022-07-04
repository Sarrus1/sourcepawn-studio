import {
  ExtensionContext,
  workspace as Workspace,
  languages,
  window,
  ProgressLocation,
  env,
} from "vscode";
import { URI } from "vscode-uri";
import { join, resolve } from "path";
import glob = require("glob");
import Parser = require("web-tree-sitter");

import { refreshDiagnostics } from "./Providers/spLinter";
import { registerSPLinter } from "./Providers/Linter/registerSPLinter";
import { registerKVLinter } from "./Providers/Linter/registerKVLinter";
import { parseSMApi } from "./Misc/parseSMAPI";
import { SP_MODE, SP_LEGENDS } from "./Misc/spConstants";
import { Providers } from "./Backend/spProviders";
import { registerSMCommands } from "./Commands/registerCommands";
import { SMDocumentFormattingEditProvider } from "./Formatters/spFormat";
import { KVDocumentFormattingEditProvider } from "./Formatters/kvFormat";
import { findMainPath, checkMainPath } from "./spUtils";
import { updateDecorations } from "./Providers/spDecorationsProvider";

export let parser: Parser;
export let spLangObj: Parser.Language;
export let symbolQuery: Parser.Query;
export let variableQuery: Parser.Query;

export function activate(context: ExtensionContext) {
  const providers = new Providers();

  const workspaceFolders = Workspace.workspaceFolders || [];
  const watcher = Workspace.createFileSystemWatcher(
    "**/*.{inc,sp}",
    false,
    true,
    false
  );

  watcher.onDidCreate((uri) => {
    const uriString = URI.file(uri.fsPath).toString();
    providers.itemsRepository.documents.set(uriString, false);
    let mainPath = findMainPath(uri);
    if (mainPath !== undefined && mainPath !== "") {
      mainPath = URI.file(mainPath).toString();
      for (const document of Workspace.textDocuments) {
        if (document.uri.toString() === mainPath) {
          refreshDiagnostics(document);
          break;
        }
      }
    }
  });
  watcher.onDidDelete((uri) => {
    providers.itemsRepository.documents.delete(uri.fsPath);
  });

  // Get all the files from the workspaces
  getDirectories(
    workspaceFolders.map((e) => e.uri.fsPath),
    providers
  );

  Workspace.onDidChangeWorkspaceFolders((e) => {
    getDirectories(
      e.added.map((folder) => folder.uri.fsPath),
      providers
    );
  });
  // Get the names and directories of optional include directories.
  let optionalIncludeDirs: string[] =
    Workspace.getConfiguration("sourcepawn").get("optionalIncludeDirsPaths") ||
    [];
  optionalIncludeDirs = optionalIncludeDirs.map((e) =>
    resolve(...workspaceFolders.map((folder) => folder.uri.fsPath), e)
  );
  getDirectories(optionalIncludeDirs, providers);

  window.withProgress(
    {
      location: ProgressLocation.Window,
      cancellable: false,
      title: "Initializing SourcePawn features",
    },
    async (progress) => {
      progress.report({ increment: 0 });

      await loadFiles(providers, context);

      progress.report({ increment: 100 });
    }
  );

  Workspace.onDidChangeConfiguration((e) => {
    if (e.affectsConfiguration("sourcepawn.MainPath")) {
      const newMainPath = findMainPath();
      if (newMainPath !== undefined && !checkMainPath(newMainPath)) {
        window.showErrorMessage(
          "A setting for the main.sp file was specified, but seems invalid. Right click on a file and use the command at the bottom of the menu to set it as main."
        );
        return;
      }
      providers.itemsRepository.documents.forEach((v, k) =>
        providers.itemsRepository.documents.set(k, false)
      );
      providers.itemsRepository.fileItems = new Map();
      loadFiles(providers, context);
    }
  });

  window.onDidChangeActiveTextEditor((e) => {
    if (e !== undefined) {
      updateDecorations(providers.itemsRepository);
      providers.itemsRepository.handleDocumentOpening(e.document.uri.fsPath);
    }
  });

  context.subscriptions.push(
    languages.registerDocumentSymbolProvider(SP_MODE, providers)
  );
  context.subscriptions.push(
    languages.registerCompletionItemProvider(
      SP_MODE,
      providers,
      "<",
      '"',
      "'",
      "/",
      "\\",
      ".",
      ":",
      " ",
      "$"
    )
  );
  context.subscriptions.push(
    languages.registerCompletionItemProvider(
      SP_MODE,
      providers.documentationProvider,
      "*"
    )
  );

  context.subscriptions.push(
    languages.registerSignatureHelpProvider(SP_MODE, providers, "(", ",", "\n")
  );

  context.subscriptions.push(
    languages.registerDocumentSemanticTokensProvider(
      SP_MODE,
      providers,
      SP_LEGENDS
    )
  );

  context.subscriptions.push(
    languages.registerDefinitionProvider(SP_MODE, providers)
  );

  context.subscriptions.push(
    languages.registerReferenceProvider(SP_MODE, providers)
  );

  context.subscriptions.push(
    languages.registerRenameProvider(SP_MODE, providers)
  );

  context.subscriptions.push(
    languages.registerDocumentFormattingEditProvider(
      {
        language: "sourcepawn",
        scheme: "file",
        pattern: "**/*.sp",
      },
      new SMDocumentFormattingEditProvider()
    )
  );

  context.subscriptions.push(
    languages.registerDocumentFormattingEditProvider(
      {
        language: "valve-kv",
      },
      new KVDocumentFormattingEditProvider()
    )
  );

  context.subscriptions.push(
    languages.registerHoverProvider(SP_MODE, providers)
  );

  context.subscriptions.push(
    languages.registerCallHierarchyProvider(SP_MODE, providers)
  );

  Workspace.onDidChangeTextDocument(
    providers.itemsRepository.handleDocumentChange,
    providers.itemsRepository,
    context.subscriptions
  );

  Workspace.onDidOpenTextDocument(
    providers.itemsRepository.handleNewDocument,
    providers.itemsRepository,
    context.subscriptions
  );

  Workspace.onDidCreateFiles(
    providers.itemsRepository.handleAddedDocument,
    providers.itemsRepository,
    context.subscriptions
  );

  // Register SM Commands
  registerSMCommands(context);

  // Register SM linter
  registerSPLinter(context);

  // Register KV linter
  registerKVLinter(context);
}

function getDirectories(paths: string[], providers: Providers) {
  for (const path of paths) {
    const files = glob.sync(path.replace(/\/\s*$/, "") + "/**/*.{inc,sp}");
    for (const file of files) {
      providers.itemsRepository.documents.set(URI.file(file).toString(), false);
    }
  }
}

async function loadFiles(providers: Providers, context: ExtensionContext) {
  console.time("build parser");
  await buildParser();
  console.timeEnd("build parser");
  console.time("parse");

  await parseSMApi(providers.itemsRepository);

  const mainPath = findMainPath();

  if (mainPath !== undefined) {
    if (!checkMainPath(mainPath)) {
      window.showErrorMessage(
        "A setting for the main .sp file was specified, but seems invalid.\
        \nRight click on a file and use the command at the bottom of the menu to set it as main."
      );
    } else {
      providers.itemsRepository.handleDocumentOpening(mainPath);
      if (window.activeTextEditor) {
        providers.itemsRepository.handleDocumentOpening(
          window.activeTextEditor.document.uri.fsPath
        );
      }
    }
  } else {
    // Load the currently opened file
    const files = await Workspace.findFiles("**/*.sp");
    if (window.activeTextEditor) {
      providers.itemsRepository.handleDocumentOpening(
        window.activeTextEditor.document.uri.fsPath
      );
    }

    const wk = Workspace.workspaceFolders;
    if (wk === undefined && files.length > 1) {
      window.showWarningMessage(
        "There are no mainpath set for this workspace.\
        The extension might not work properly.\
        \nRight click on a file and `Set file as main path`."
      );
      return;
    }

    if (
      files.length > 1 &&
      !context.workspaceState.get("sp-mainpath-dontshowagain")
    ) {
      window
        .showWarningMessage(
          "There is no mainpath set for this workspace. The extension will not work properly.",
          "Select a main path",
          "Learn more",
          "Don't show again"
        )
        .then((v) => {
          if (v === "Select a main path") {
            window.showQuickPick(files.map((e) => e.fsPath)).then(async (v) => {
              await Workspace.getConfiguration("sourcepawn", wk[0]).update(
                "MainPath",
                v
              );
            });
          } else if (v === "Learn more") {
            env.openExternal(
              URI.parse(
                "https://github.com/Sarrus1/sourcepawn-vscode/wiki#setting-up-your-project"
              )
            );
          } else if (v === "Don't show again") {
            context.workspaceState.update("sp-mainpath-dontshowagain", true);
          }
        });
    }
  }

  updateDecorations(providers.itemsRepository);

  console.timeEnd("parse");
}

async function buildParser() {
  await Parser.init();
  parser = new Parser();
  const langFile = join(__dirname, "tree-sitter-sourcepawn.wasm");
  spLangObj = await Parser.Language.load(langFile);
  parser.setLanguage(spLangObj);
  variableQuery = spLangObj.query(
    "[(variable_declaration_statement) @declaration.variable (old_variable_declaration_statement)  @declaration.variable]"
  );
  symbolQuery = spLangObj.query("[(symbol) @symbol (this) @symbol]");
}
