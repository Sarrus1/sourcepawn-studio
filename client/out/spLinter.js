"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.registerSMLinter = exports.textDocumentClosed = exports.textDocumentChanged = exports.textDocumentOpened = exports.activeEditorChanged = exports.compilerDiagnostics = exports.refreshDiagnostics = exports.throttles = exports.TimeoutFunction = void 0;
const vscode = require("vscode");
const path = require("path");
const fs = require("fs");
const child_process_1 = require("child_process");
const vscode_uri_1 = require("vscode-uri");
let myExtDir = vscode.extensions.getExtension("Sarrus.sourcepawn-vscode").extensionPath;
let TempPath = path.join(myExtDir, "tmpCompiled.smx");
const tempFile = path.join(__dirname, "temp.sp");
class TimeoutFunction {
    constructor() {
        this.timeout = undefined;
    }
    start(callback, delay) {
        this.timeout = setTimeout(callback, delay);
    }
    cancel() {
        if (this.timeout) {
            clearTimeout(this.timeout);
            this.timeout = undefined;
        }
    }
}
exports.TimeoutFunction = TimeoutFunction;
exports.throttles = {};
function refreshDiagnostics(document, compilerDiagnostics) {
    const DocumentDiagnostics = new Map();
    // Check if the user specified not to enable the linter for this file
    const start = new vscode.Position(0, 0);
    const end = new vscode.Position(1, 0);
    const range = new vscode.Range(start, end);
    const text = document.getText(range);
    if (text == "" || /\/\/linter=false/.test(text)) {
        return ReturnNone(document.uri);
    }
    const spcomp = vscode.workspace
        .getConfiguration("sourcepawnLanguageServer")
        .get("spcomp_path") || "";
    if (!vscode.workspace
        .getConfiguration("sourcepawnLanguageServer")
        .get("spcomp_path") ||
        (spcomp !== "" && !fs.existsSync(spcomp))) {
        vscode.window
            .showErrorMessage("SourceMod compiler not found in the project. You need to set the spcomp path for the Linter to work.", "Open Settings")
            .then((choice) => {
            if (choice === "Open Settings") {
                vscode.commands.executeCommand("workbench.action.openWorkspaceSettings");
            }
        });
    }
    let throttle = exports.throttles[document.uri.path];
    if (throttle === undefined) {
        throttle = new TimeoutFunction();
        exports.throttles[document.uri.path] = throttle;
    }
    throttle.cancel();
    throttle.start(function () {
        var _a;
        let filename = document.fileName;
        let MainPath = vscode.workspace.getConfiguration("sourcepawnLanguageServer").get("main_path") || "";
        if (MainPath != "") {
            try {
                let workspace = vscode.workspace.workspaceFolders[0];
                MainPath = path.join(workspace.uri.fsPath, MainPath);
                filename = path.basename(MainPath);
            }
            catch (error) {
                ReturnNone(document.uri);
                vscode.window
                    .showErrorMessage("A setting for the main.sp file was specified, but seems invalid. Please make sure it is valid.", "Open Settings").then((choice) => {
                    if (choice === "Open Settings") {
                        vscode.commands.executeCommand("workbench.action.openWorkspaceSettings");
                    }
                });
            }
        }
        if (path.extname(filename) === ".sp") {
            let scriptingFolder;
            let filePath;
            try {
                if (MainPath != "") {
                    scriptingFolder = path.dirname(MainPath);
                    filePath = MainPath;
                }
                else {
                    scriptingFolder = path.dirname(document.uri.fsPath);
                    let file = fs.openSync(tempFile, "w", 0o765);
                    fs.writeSync(file, document.getText());
                    fs.closeSync(file);
                    filePath = tempFile;
                }
                let spcomp_opt = [
                    "-i" +
                        vscode.workspace
                            .getConfiguration("sourcepawnLanguageServer")
                            .get("sourcemod_home") || "",
                    "-i" + path.join(scriptingFolder, "include"),
                    "-v0",
                    filePath,
                    "-o" + TempPath,
                ];
                let compilerOptions = vscode.workspace.getConfiguration("sourcepawnLanguageServer")
                    .get("linterCompilerOptions");
                // Add a space at the beginning of every element, for security.
                for (let i = 0; i < compilerOptions.length; i++) {
                    spcomp_opt.push(" " + compilerOptions[i]);
                }
                let includes_dirs = vscode.workspace
                    .getConfiguration("sourcepawnLanguageServer")
                    .get("optionalIncludeDirsPaths");
                // Add the optional includes folders.
                for (let includes_dir of includes_dirs) {
                    if (includes_dir != "") {
                        spcomp_opt.push("-i" + includes_dir);
                    }
                }
                // Run the blank compile.
                child_process_1.execFileSync(spcomp, spcomp_opt);
                fs.unlinkSync(TempPath);
            }
            catch (error) {
                let regex = /([\/A-z-_0-9. ]*)\((\d+)+\) : ((error|fatal error|warning).+)/gm;
                let matches;
                let path;
                let diagnostics;
                let range;
                let severity;
                while ((matches = regex.exec(((_a = error.stdout) === null || _a === void 0 ? void 0 : _a.toString()) || ""))) {
                    range = new vscode.Range(new vscode.Position(Number(matches[2]) - 1, 0), new vscode.Position(Number(matches[2]) - 1, 256));
                    severity =
                        matches[4] === "warning"
                            ? vscode.DiagnosticSeverity.Warning
                            : vscode.DiagnosticSeverity.Error;
                    path = MainPath != "" ? matches[1] : document.uri.fsPath;
                    if (DocumentDiagnostics.has(path)) {
                        diagnostics = DocumentDiagnostics.get(path);
                    }
                    else {
                        diagnostics = [];
                    }
                    diagnostics.push(new vscode.Diagnostic(range, matches[3], severity));
                    DocumentDiagnostics.set(path, diagnostics);
                }
            }
            compilerDiagnostics.clear();
            for (let [path, diagnostics] of DocumentDiagnostics) {
                compilerDiagnostics.set(vscode_uri_1.URI.parse(path), diagnostics);
            }
        }
    }, 300);
}
exports.refreshDiagnostics = refreshDiagnostics;
function ReturnNone(uri) {
    let diagnostics = [];
    return exports.compilerDiagnostics.set(uri, diagnostics);
}
exports.compilerDiagnostics = vscode.languages.createDiagnosticCollection("compiler");
exports.activeEditorChanged = vscode.window.onDidChangeActiveTextEditor((editor) => {
    if (editor) {
        refreshDiagnostics(editor.document, exports.compilerDiagnostics);
    }
});
exports.textDocumentOpened = vscode.workspace.onDidOpenTextDocument((event) => {
    refreshDiagnostics(event, exports.compilerDiagnostics);
});
exports.textDocumentChanged = vscode.workspace.onDidChangeTextDocument((event) => {
    refreshDiagnostics(event.document, exports.compilerDiagnostics);
});
exports.textDocumentClosed = vscode.workspace.onDidCloseTextDocument((document) => {
    exports.compilerDiagnostics.delete(document.uri);
    delete exports.throttles[document.uri.path];
});
function registerSMLinter(context) {
    context.subscriptions.push(exports.compilerDiagnostics);
    context.subscriptions.push(exports.activeEditorChanged);
    context.subscriptions.push(exports.textDocumentChanged);
    context.subscriptions.push(exports.textDocumentClosed);
}
exports.registerSMLinter = registerSMLinter;
//# sourceMappingURL=spLinter.js.map