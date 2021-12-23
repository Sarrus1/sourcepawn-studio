import {
  ExtensionContext,
  workspace as Workspace,
  languages,
  window,
  commands,
} from "vscode";
import {
  registerSMLinter,
  compilerDiagnostics,
  refreshDiagnostics,
} from "./spLinter";
const glob = require("glob");
import { SP_MODE, SP_LEGENDS } from "./Misc/spConstants";
import { Providers } from "./Providers/spProviders";
import { registerSMCommands } from "./Commands/registerCommands";
import { SMDocumentFormattingEditProvider } from "./Formatters/spFormat";
import { CFGDocumentFormattingEditProvider } from "./Formatters/cfgFormat";
import { URI } from "vscode-uri";
import { findMainPath } from "./spUtils";
import { resolve } from "path";

export function activate(context: ExtensionContext) {
  const providers = new Providers(context.globalState);
  providers.parseSMApi();
  let workspaceFolders = Workspace.workspaceFolders;
  if (workspaceFolders === undefined) {
    window.showWarningMessage(
      "No workspace or folder found. \n Please open the folder containing your .sp file, not just the .sp file."
    );
  } else {
    let watcher = Workspace.createFileSystemWatcher(
      "**​/*.{inc,sp}",
      false,
      true,
      false
    );

    watcher.onDidCreate((uri) => {
      let uriString = URI.file(uri.fsPath).toString();
      providers.itemsRepository.documents.add(uriString);
      let mainPath = findMainPath(uri);
      if (mainPath !== undefined) {
        mainPath = URI.file(mainPath).toString();
        for (let document of Workspace.textDocuments) {
          if (document.uri.toString() === mainPath) {
            refreshDiagnostics(document, compilerDiagnostics);
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
  let optionalIncludeDirs: string[] = Workspace.getConfiguration(
    "sourcepawn"
  ).get("optionalIncludeDirsPaths");
  optionalIncludeDirs = optionalIncludeDirs.map((e) =>
    resolve(...workspaceFolders.map((folder) => folder.uri.fsPath), e)
  );
  getDirectories(optionalIncludeDirs, providers);

  let mainPath: string = findMainPath();
  if (mainPath !== undefined && mainPath != "") {
    providers.handle_document_opening(mainPath);
  } else if (mainPath == "") {
    window
      .showErrorMessage(
        "A setting for the main.sp file was specified, but seems invalid. Please make sure it is valid.",
        "Open Settings"
      )
      .then((choice) => {
        if (choice === "Open Settings") {
          commands.executeCommand(
            "workbench.action.openSettings",
            "@ext:sarrus.sourcepawn-vscode"
          );
        }
      });
  }

  // Load the currently opened file
  if (window.activeTextEditor != undefined) {
    providers.handle_document_opening(
      window.activeTextEditor.document.uri.fsPath
    );
  }
  window.onDidChangeActiveTextEditor((e) => {
    if (e !== undefined) {
      providers.handle_document_opening(e.document.uri.fsPath);
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
    providers.handleDocumentChange,
    providers,
    context.subscriptions
  );
  Workspace.onDidOpenTextDocument(
    providers.handleNewDocument,
    providers,
    context.subscriptions
  );
  Workspace.onDidCreateFiles(
    providers.handleAddedDocument,
    providers,
    context.subscriptions
  );

  // Register SM Commands
  registerSMCommands(context);

  // Register SM linter
  registerSMLinter(context);
}

function getDirectories(paths: string[], providers: Providers) {
  for (let path of paths) {
    let files = glob.sync(path.replace(/\/\s*$/, "") + "/**/*.{inc,sp}");
    for (let file of files) {
      providers.itemsRepository.documents.add(URI.file(file).toString());
    }
  }
}
