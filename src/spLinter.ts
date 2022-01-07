import {
  extensions,
  TextDocument,
  DiagnosticCollection,
  Diagnostic,
  workspace as Workspace,
  Position,
  Range,
  window,
  Uri,
  languages,
  ExtensionContext,
} from "vscode";
import { openSync, writeSync, unlink, closeSync } from "fs";
import { join, extname, dirname } from "path";
import { execFile } from "child_process";

import { getAllPossibleIncludeFolderPaths } from "./Backend/spFileHandlers";
import { parseSPCompErrors } from "./Misc/parseSPCompErrors";
import { findMainPath } from "./spUtils";

let myExtDir: string = extensions.getExtension("Sarrus.sourcepawn-vscode")
  .extensionPath;
let TempPath: string = join(myExtDir, "tmpCompiled.smx");

const tempFile = join(__dirname, "temp.sp");

export class TimeoutFunction {
  private timeout;

  constructor() {
    this.timeout = undefined;
  }

  public start(callback: (...args: any[]) => void, delay: number) {
    this.timeout = setTimeout(callback, delay);
  }

  public cancel() {
    if (this.timeout) {
      clearTimeout(this.timeout);
      this.timeout = undefined;
    }
  }
}

export let throttles: { [key: string]: TimeoutFunction } = {};

export function refreshDiagnostics(
  document: TextDocument,
  compilerDiagnostics: DiagnosticCollection
) {
  // Check if the user specified not to enable the linter for this file.
  const start = new Position(0, 0);
  const end = new Position(1, 0);
  const range = new Range(start, end);
  const text = document.getText(range);

  // Check if the setting to activate the linter is set to true.
  const workspaceFolder = Workspace.getWorkspaceFolder(document.uri);
  const enableLinter: boolean = Workspace.getConfiguration(
    "sourcepawn",
    workspaceFolder
  ).get<boolean>("enableLinter");

  // Stop early if linter is disabled.
  if (text === "" || /\/\/linter=false/.test(text) || !enableLinter) {
    return returnNone(document.uri);
  }

  const spcomp =
    Workspace.getConfiguration("sourcepawn", workspaceFolder).get<string>(
      "SpcompPath"
    ) || "";

  let throttle = throttles[document.uri.path];
  if (throttle === undefined) {
    throttle = new TimeoutFunction();
    throttles[document.uri.path] = throttle;
  }

  throttle.cancel();
  throttle.start(() => {
    let filename: string = document.fileName;
    if (extname(filename) !== ".sp") {
      return;
    }
    const MainPath = findMainPath(document.uri);

    let scriptingFolder: string;
    let filePath: string;
    if (MainPath !== undefined) {
      scriptingFolder = dirname(MainPath);
      filePath = MainPath;
    } else {
      scriptingFolder = dirname(document.uri.fsPath);
      let file = openSync(tempFile, "w", 0o765);
      writeSync(file, document.getText());
      closeSync(file);
      filePath = tempFile;
    }
    let spcomp_opt: string[] = [
      "-i" +
        Workspace.getConfiguration("sourcepawn", workspaceFolder).get(
          "SourcemodHome"
        ) || "",
      "-i" + scriptingFolder,
      "-i" + join(scriptingFolder, "include"),
      "-v0",
      filePath,
      "-o" + TempPath,
    ];
    let compilerOptions: string[] = Workspace.getConfiguration(
      "sourcepawn",
      workspaceFolder
    ).get("linterCompilerOptions");
    // Add a space at the beginning of every element, for security.
    for (let i = 0; i < compilerOptions.length; i++) {
      spcomp_opt.push(" " + compilerOptions[i]);
    }

    // Add the optional includes folders.
    getAllPossibleIncludeFolderPaths(document.uri, true).forEach((e) =>
      spcomp_opt.push(`-i${e}`)
    );

    // Run the blank compile.
    execFile(spcomp, spcomp_opt, (error, stdout) => {
      // If it compiled successfully, unlink the temporary files.
      if (!error) {
        unlink(TempPath, (err) => {
          if (err) {
            console.error(err);
          }
        });
      }
      parseSPCompErrors(stdout, compilerDiagnostics, document.uri.fsPath);
    });
  }, 300);
}

function returnNone(uri: Uri) {
  let diagnostics: Diagnostic[] = [];
  return compilerDiagnostics.set(uri, diagnostics);
}

export let compilerDiagnostics = languages.createDiagnosticCollection(
  "compiler"
);

export let activeEditorChanged = window.onDidChangeActiveTextEditor(
  (editor) => {
    if (editor) {
      refreshDiagnostics(editor.document, compilerDiagnostics);
    }
  }
);

export let textDocumentClosed = Workspace.onDidCloseTextDocument((document) => {
  compilerDiagnostics.delete(document.uri);
  delete throttles[document.uri.path];
});

export function registerSMLinter(context: ExtensionContext) {
  context.subscriptions.push(compilerDiagnostics);
  context.subscriptions.push(activeEditorChanged);
  context.subscriptions.push(textDocumentClosed);
}
