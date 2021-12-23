import {
  extensions,
  TextDocument,
  DiagnosticCollection,
  Diagnostic,
  workspace as Workspace,
  WorkspaceFolder,
  Position,
  Range,
  window,
  DiagnosticSeverity,
  commands,
  Uri,
  languages,
  ExtensionContext,
} from "vscode";
import { existsSync, openSync, writeSync, unlink, closeSync } from "fs";
import { join, basename, extname, dirname, resolve } from "path";
import { execFile } from "child_process";
import { URI } from "vscode-uri";
import { errorDetails } from "./Misc/errorMessages";

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
  const DocumentDiagnostics: Map<string, Diagnostic[]> = new Map();
  // Check if the user specified not to enable the linter for this file
  const start = new Position(0, 0);
  const end = new Position(1, 0);
  const range = new Range(start, end);
  const text: string = document.getText(range);

  let workspaceFolder = Workspace.getWorkspaceFolder(document.uri);
  const enableLinter: boolean = Workspace.getConfiguration(
    "sourcepawn",
    workspaceFolder
  ).get<boolean>("enableLinter");

  if (text == "" || /\/\/linter=false/.test(text) || !enableLinter) {
    return ReturnNone(document.uri);
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
  throttle.start(function () {
    let filename: string = document.fileName;
    let MainPath: string =
      Workspace.getConfiguration("sourcepawn", workspaceFolder).get(
        "MainPath"
      ) || "";
    if (MainPath != "") {
      try {
        if (!existsSync(MainPath)) {
          let workspace: WorkspaceFolder = Workspace.workspaceFolders[0];
          MainPath = join(workspace.uri.fsPath, MainPath);
          if (!existsSync(MainPath)) {
            throw new Error("MainPath is incorrect.");
          }
        }
        filename = basename(MainPath);
      } catch (error) {
        ReturnNone(document.uri);
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
    }
    let extName = extname(filename);
    if (extName === ".sp") {
      let scriptingFolder: string;
      let filePath: string;
      try {
        if (MainPath != "") {
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
        /*
        let includes_dirs: string[] = Workspace.getConfiguration(
          "sourcepawn",
          workspaceFolder
        ).get("optionalIncludeDirsPaths");
        // Add the optional includes folders.
        for (let includes_dir of includes_dirs) {
          if (includes_dir != "") {
            spcomp_opt.push(
              "-i" +
                resolve(
                  Workspace.workspaceFolders.map(
                    (folder) => folder.uri.fsPath
                  ) + includes_dir
                )
            );
          }
        }*/

        // Add the optional includes folders.
        let optionalIncludeDirs: string[] = Workspace.getConfiguration(
          "sourcepawn",
          workspaceFolder
        ).get("optionalIncludeDirsPaths");
        optionalIncludeDirs = optionalIncludeDirs.map((e) =>
          resolve(workspaceFolder.uri.fsPath, e)
        );
        for (let includeDir of optionalIncludeDirs) {
          if (includeDir !== "") {
            spcomp_opt.push("-i" + includeDir);
          }
        }
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
          const re = /([:\/\\A-Za-z\-_0-9. ]*)\((\d+)+\) : ((error|fatal error|warning) ([0-9]*)):\s+(.*)/gm;
          let matches: RegExpExecArray | null;
          let path: string;
          let diagnostics: Diagnostic[];
          let range: Range;
          let severity: DiagnosticSeverity;
          do {
            matches = re.exec(stdout.toString() || "");
            if (matches) {
              range = new Range(
                new Position(Number(matches[2]) - 1, 0),
                new Position(Number(matches[2]) - 1, 256)
              );
              severity =
                matches[4] === "warning"
                  ? DiagnosticSeverity.Warning
                  : DiagnosticSeverity.Error;
              path = MainPath != "" ? matches[1] : document.uri.fsPath;
              if (DocumentDiagnostics.has(path)) {
                diagnostics = DocumentDiagnostics.get(path);
              } else {
                diagnostics = [];
              }
              let message: string = GenerateDetailedError(
                matches[5],
                matches[6]
              );
              let diagnostic: Diagnostic = new Diagnostic(
                range,
                message,
                severity
              );
              diagnostics.push(diagnostic);
              DocumentDiagnostics.set(path, diagnostics);
            }
          } while (matches);
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
  if (errorDetails[errorCode] !== undefined) {
    errorMsg += "\n\n" + errorDetails[errorCode];
  }
  return errorMsg;
}

function ReturnNone(uri: Uri) {
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

export let textDocumentOpened = Workspace.onDidOpenTextDocument((event) => {
  refreshDiagnostics(event, compilerDiagnostics);
});

export let textDocumentChanged = Workspace.onDidChangeTextDocument((event) => {
  refreshDiagnostics(event.document, compilerDiagnostics);
});

export let textDocumentClosed = Workspace.onDidCloseTextDocument((document) => {
  compilerDiagnostics.delete(document.uri);
  delete throttles[document.uri.path];
});

export function registerSMLinter(context: ExtensionContext) {
  context.subscriptions.push(compilerDiagnostics);
  context.subscriptions.push(activeEditorChanged);
  context.subscriptions.push(textDocumentChanged);
  context.subscriptions.push(textDocumentClosed);
}
