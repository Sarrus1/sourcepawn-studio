import {
  ExtensionContext,
  workspace as Workspace,
  languages,
  window,
  StatusBarAlignment,
  StatusBarItem,
  TextDocumentChangeEvent,
} from "vscode";
import { URI } from "vscode-uri";
import { resolve } from "path";
const glob = require("glob");

import { refreshDiagnostics } from "./Providers/spLinter";
import { registerSPLinter } from "./Providers/Linter/registerSPLinter";
import { registerCFGLinter } from "./Providers/Linter/registerCFGLinter";
import { parseSMApi } from "./Misc/parseSMAPI";
import { SP_MODE, SP_LEGENDS } from "./Misc/spConstants";
import { Providers } from "./Backend/spProviders";
import { registerSMCommands } from "./Commands/registerCommands";
import { SMDocumentFormattingEditProvider } from "./Formatters/spFormat";
import { CFGDocumentFormattingEditProvider } from "./Formatters/cfgFormat";
import { findMainPath, checkMainPath } from "./spUtils";
import { updateDecorations } from "./Providers/decorationsProvider";

export function activate(context: ExtensionContext) {
  const providers = new Providers(context.globalState);

  const SBItem = window.createStatusBarItem(StatusBarAlignment.Left, 0);
  SBItem.command = "status.enablingSPFeatures";
  SBItem.text = "Enabling SourcePawn features...";
  SBItem.show();

  let workspaceFolders = Workspace.workspaceFolders || [];
  if (workspaceFolders.length === 0) {
    window.showWarningMessage(
      "No workspace or folder found. \n Please open the folder containing your .sp file, not just the .sp file."
    );
  } else {
    const watcher = Workspace.createFileSystemWatcher(
      "**/*.{inc,sp}",
      false,
      true,
      false
    );

    watcher.onDidCreate((uri) => {
      let uriString = URI.file(uri.fsPath).toString();
      providers.itemsRepository.documents.add(uriString);
      let mainPath = findMainPath(uri);
      if (mainPath !== undefined && mainPath !== "") {
        mainPath = URI.file(mainPath).toString();
        for (let document of Workspace.textDocuments) {
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
  }

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

  loadFiles(providers, SBItem);

  Workspace.onDidChangeConfiguration((e) => {
    if (e.affectsConfiguration("sourcepawn.MainPath")) {
      let newMainPath = findMainPath();
      if (newMainPath !== undefined && !checkMainPath(newMainPath)) {
        window.showErrorMessage(
          "A setting for the main.sp file was specified, but seems invalid. Right click on a file and use the command at the bottom of the menu to set it as main."
        );
      }
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
      " "
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
      [
        {
          language: "sp-translations",
        },
        {
          language: "sp-gamedata",
        },
        {
          language: "valve-cfg",
        },
        {
          language: "valve-ini",
        },
        {
          language: "sourcemod-kv",
        },
      ],
      new CFGDocumentFormattingEditProvider()
    )
  );

  context.subscriptions.push(
    languages.registerHoverProvider(SP_MODE, providers)
  );

  Workspace.onDidChangeTextDocument(
    throttle(providers.itemsRepository.handleDocumentChange, 50),
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

  // Register CFG linter
  registerCFGLinter(context);
}

function getDirectories(paths: string[], providers: Providers) {
  for (let path of paths) {
    let files = glob.sync(path.replace(/\/\s*$/, "") + "/**/*.{inc,sp}");
    for (let file of files) {
      providers.itemsRepository.documents.add(URI.file(file).toString());
    }
  }
}

async function loadFiles(providers: Providers, SBItem: StatusBarItem) {
  console.time("parse");
  await parseSMApi(providers.itemsRepository);

  const mainPath = findMainPath();
  if (mainPath !== undefined) {
    if (!checkMainPath(mainPath)) {
      window.showErrorMessage(
        "A setting for the main.sp file was specified, but seems invalid. Right click on a file and use the command at the bottom of the menu to set it as main."
      );
    } else {
      providers.itemsRepository.handleDocumentOpening(mainPath);
    }
  }

  // Load the currently opened file
  if (window.activeTextEditor != undefined) {
    providers.itemsRepository.handleDocumentOpening(
      window.activeTextEditor.document.uri.fsPath
    );
  }
  updateDecorations(providers.itemsRepository);

  SBItem.hide();
  console.timeEnd("parse");
}

function throttle(
  callback: (event: TextDocumentChangeEvent) => void,
  limit: number
) {
  let waiting = false;
  return function () {
    if (!waiting) {
      callback.apply(this, arguments);
      waiting = true;
      setTimeout(function () {
        waiting = false;
      }, limit);
    }
  };
}
