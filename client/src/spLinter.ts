import * as vscode from "vscode";
import * as path from "path";
import * as fs from "fs";
import { execFile } from "child_process";
import { URI } from "vscode-uri";
import { errorDetails } from "./spIndex";

let myExtDir: string = vscode.extensions.getExtension(
  "Sarrus.sourcepawn-vscode"
).extensionPath;
let TempPath: string = path.join(myExtDir, "tmpCompiled.smx");

const tempFile = path.join(__dirname, "temp.sp");

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
  document: vscode.TextDocument,
  compilerDiagnostics: vscode.DiagnosticCollection
) {
  const DocumentDiagnostics: Map<string, vscode.Diagnostic[]> = new Map();
  // Check if the user specified not to enable the linter for this file
  const start = new vscode.Position(0, 0);
  const end = new vscode.Position(1, 0);
  const range = new vscode.Range(start, end);
  const text: string = document.getText(range);
  const enableLinter: boolean = vscode.workspace
    .getConfiguration("sourcepawn")
    .get<boolean>("enableLinter");
  if (text == "" || /\/\/linter=false/.test(text) || !enableLinter) {
    return ReturnNone(document.uri);
  }
  const spcomp =
    vscode.workspace.getConfiguration("sourcepawn").get<string>("SpcompPath") ||
    "";

  let throttle = throttles[document.uri.path];
  if (throttle === undefined) {
    throttle = new TimeoutFunction();
    throttles[document.uri.path] = throttle;
  }

  throttle.cancel();
  throttle.start(function () {
    let filename: string = document.fileName;
    let MainPath: string =
      vscode.workspace.getConfiguration("sourcepawn").get("MainPath") || "";
    if (MainPath != "") {
      try {
        if (!fs.existsSync(MainPath)) {
          let workspace: vscode.WorkspaceFolder =
            vscode.workspace.workspaceFolders[0];
          MainPath = path.join(workspace.uri.fsPath, MainPath);
          if (!fs.existsSync(MainPath)) {
            throw "MainPath is incorrect.";
          }
        }
        filename = path.basename(MainPath);
      } catch (error) {
        ReturnNone(document.uri);
        vscode.window
          .showErrorMessage(
            "A setting for the main.sp file was specified, but seems invalid. Please make sure it is valid.",
            "Open Settings"
          )
          .then((choice) => {
            if (choice === "Open Settings") {
              vscode.commands.executeCommand(
                "workbench.action.openWorkspaceSettings"
              );
            }
          });
      }
    }
		let extName = path.extname(filename);
    if (extName === ".sp") {
      let scriptingFolder: string;
      let filePath: string;
      try {
        if (MainPath != "") {
          scriptingFolder = path.dirname(MainPath);
          filePath = MainPath;
        } else {
          scriptingFolder = path.dirname(document.uri.fsPath);
          let file = fs.openSync(tempFile, "w", 0o765);
          fs.writeSync(file, document.getText());
          fs.closeSync(file);
          filePath = tempFile;
        }
        let spcomp_opt: string[] = [
          "-i" +
            vscode.workspace
              .getConfiguration("sourcepawn")
              .get("SourcemodHome") || "",
          "-i" + path.join(scriptingFolder, "include"),
          "-v0",
          filePath,
          "-o" + TempPath,
        ];
        let compilerOptions: string[] = vscode.workspace
          .getConfiguration("sourcepawn")
          .get("linterCompilerOptions");
        // Add a space at the beginning of every element, for security.
        for (let i = 0; i < compilerOptions.length; i++) {
          spcomp_opt.push(" " + compilerOptions[i]);
        }
        let includes_dirs: string[] = vscode.workspace
          .getConfiguration("sourcepawn")
          .get("optionalIncludeDirsPaths");
        // Add the optional includes folders.
        for (let includes_dir of includes_dirs) {
          if (includes_dir != "") {
            spcomp_opt.push("-i" + includes_dir);
          }
        }
        // Run the blank compile.
        execFile(spcomp, spcomp_opt, (error, stdout) => {
          // If it compiled successfully, unlink the temporary files.
          if (!error) {
            fs.unlink(TempPath, (err) => {
              if (err) {
                console.error(err);
              }
            });
          }
          let regex = /([:\/\\A-Za-z\-_0-9. ]*)\((\d+)+\) : ((error|fatal error|warning) ([0-9]*)):\s+(.*)/gm;
          let matches: RegExpExecArray | null;
          let path: string;
          let diagnostics: vscode.Diagnostic[];
          let range: vscode.Range;
          let severity: vscode.DiagnosticSeverity;
          while ((matches = regex.exec(stdout.toString() || ""))) {
            range = new vscode.Range(
              new vscode.Position(Number(matches[2]) - 1, 0),
              new vscode.Position(Number(matches[2]) - 1, 256)
            );
            severity =
              matches[4] === "warning"
                ? vscode.DiagnosticSeverity.Warning
                : vscode.DiagnosticSeverity.Error;
            path = MainPath != "" ? matches[1] : document.uri.fsPath;
            if (DocumentDiagnostics.has(path)) {
              diagnostics = DocumentDiagnostics.get(path);
            } else {
              diagnostics = [];
            }
            let message: string = GenerateDetailedError(matches[5], matches[6]);
            let diagnostic: vscode.Diagnostic = new vscode.Diagnostic(
              range,
              message,
              severity
            );
            diagnostics.push(diagnostic);
            DocumentDiagnostics.set(path, diagnostics);
          }
          compilerDiagnostics.clear();
          for (let [path, diagnostics] of DocumentDiagnostics) {
            compilerDiagnostics.set(URI.file(path), diagnostics);
          }
        });
      } catch (err) {
        console.error(err);
      }
    }
  }, 300);
}

function GenerateDetailedError(errorCode: string, errorMsg: string): string {
  if (typeof errorDetails[errorCode] != "undefined") {
    errorMsg += "\n\n" + errorDetails[errorCode];
  }
	return errorMsg;
}

function ReturnNone(uri: vscode.Uri) {
  let diagnostics: vscode.Diagnostic[] = [];
  return compilerDiagnostics.set(uri, diagnostics);
}

export let compilerDiagnostics = vscode.languages.createDiagnosticCollection(
  "compiler"
);

export let activeEditorChanged = vscode.window.onDidChangeActiveTextEditor(
  (editor) => {
    if (editor) {
      refreshDiagnostics(editor.document, compilerDiagnostics);
    }
  }
);

export let textDocumentOpened = vscode.workspace.onDidOpenTextDocument(
  (event) => {
    refreshDiagnostics(event, compilerDiagnostics);
  }
);

export let textDocumentChanged = vscode.workspace.onDidChangeTextDocument(
  (event) => {
    refreshDiagnostics(event.document, compilerDiagnostics);
  }
);

export let textDocumentClosed = vscode.workspace.onDidCloseTextDocument(
  (document) => {
    compilerDiagnostics.delete(document.uri);
    delete throttles[document.uri.path];
  }
);

export function registerSMLinter(context: vscode.ExtensionContext) {
  context.subscriptions.push(compilerDiagnostics);
  context.subscriptions.push(activeEditorChanged);
  context.subscriptions.push(textDocumentChanged);
  context.subscriptions.push(textDocumentClosed);
}
