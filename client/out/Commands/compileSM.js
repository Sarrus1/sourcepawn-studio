"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.run = void 0;
const vscode = require("vscode");
const path = require("path");
const fs = require("fs");
const os = require("os");
async function run(args) {
    var _a;
    let activeDocumentPath = vscode.workspace.getConfiguration("sourcepawnLanguageServer").get("main_path") || "";
    if (activeDocumentPath != "") {
        try {
            let workspace = vscode.workspace.workspaceFolders[0];
            activeDocumentPath = path.join(workspace.uri.fsPath, activeDocumentPath);
        }
        catch (error) {
            vscode.window
                .showErrorMessage("A setting for the main.sp file was specified, but seems invalid. Please make sure it is valid.", "Open Settings").then((choice) => {
                if (choice === "Open Settings") {
                    vscode.commands.executeCommand("workbench.action.openWorkspaceSettings");
                }
            });
            return;
        }
    }
    else {
        activeDocumentPath = vscode.window.activeTextEditor.document.uri.fsPath;
    }
    let scriptingPath = path.dirname(activeDocumentPath);
    let activeDocumentName = path.basename(activeDocumentPath);
    activeDocumentName = activeDocumentName.replace(".sp", ".smx");
    let activeDocumentExt = path.extname(activeDocumentPath);
    // Don't compile if it's not a .sp file.
    if (activeDocumentExt != ".sp") {
        vscode.window.showErrorMessage("Not a .sp file, aborting");
        return;
    }
    const spcomp = vscode.workspace.getConfiguration("sourcepawnLanguageServer").get("spcomp_path") || "";
    if (!spcomp) {
        vscode.window
            .showErrorMessage("SourceMod compiler not found in the project. You need to set the spcomp path for the Linter to work.", "Open Settings")
            .then((choice) => {
            if (choice === "Open Settings") {
                vscode.commands.executeCommand("workbench.action.openWorkspaceSettings");
            }
        });
        return;
    }
    // Open a terminal window
    let terminals = vscode.window.terminals;
    let terminal;
    // Try to open current terminal window instead of opening a new one.
    if (!terminals) {
        terminal = vscode.window.createTerminal("SourcePawn compile");
    }
    else {
        let found = false;
        for (let terminal_elt of terminals) {
            if (terminal_elt.name.includes("SourcePawn compile")) {
                terminal = terminal_elt;
                found = true;
                break;
            }
        }
        if (!found) {
            terminal = vscode.window.createTerminal("SourcePawn compile");
        }
    }
    terminal.show();
    let workspaceFolderPath = ((_a = vscode.workspace.workspaceFolders) === null || _a === void 0 ? void 0 : _a[0].uri.fsPath) || "";
    // Create plugins folder if it doesn't exist.
    let pluginsFolderPath = path.join(workspaceFolderPath, "plugins/");
    if (!fs.existsSync(pluginsFolderPath)) {
        fs.mkdirSync(pluginsFolderPath);
    }
    let command = (os.platform() == 'win32' ? "." : "").concat(
    // Compiler path
    "\'" +
        spcomp +
        "\'", 
    // Seperate compiler and script path
    " ", 
    // Script path (script to compile)
    "\'" +
        activeDocumentPath +
        "\'", 
    // Output path for the smx file
    " -o=" +
        "\'" +
        pluginsFolderPath + activeDocumentName +
        "\'", 
    // Set the path for sm_home
    " -i=" +
        "\'", vscode.workspace.getConfiguration("sourcepawnLanguageServer").get("sourcemod_home") || "", "\'", " -i=" +
        "\'", scriptingPath + "/include" || "", "\'");
    let compilerOptions = vscode.workspace.getConfiguration("sourcepawnLanguageServer")
        .get("compilerOptions");
    // Add a space at the beginning of every element, for security.
    for (let i = 0; i < compilerOptions.length; i++) {
        command += (" " + compilerOptions[i]);
    }
    let includes_dirs = vscode.workspace
        .getConfiguration("sourcepawnLanguageServer")
        .get("optionalIncludeDirsPaths");
    // Add the optional includes folders.
    for (let includes_dir of includes_dirs) {
        if (includes_dir != "") {
            command += (" -i=" + "\'" + includes_dir + "\'");
        }
    }
    try {
        terminal.sendText(command);
    }
    catch (error) {
        console.log(error);
    }
}
exports.run = run;
//# sourceMappingURL=compileSM.js.map