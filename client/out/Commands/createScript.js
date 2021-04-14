"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.run = void 0;
const vscode = require("vscode");
const fs = require("fs");
const path = require("path");
function run(args) {
    let author_name = vscode.workspace.getConfiguration("sourcepawnLanguageServer").get("author_name");
    if (!author_name) {
        vscode.window
            .showWarningMessage("You didn't specify an author name.", "Open Settings")
            .then((choice) => {
            if (choice === "Open Settings") {
                vscode.commands.executeCommand("workbench.action.openWorkspaceSettings");
            }
        });
    }
    let github_name = vscode.workspace.getConfiguration("sourcepawnLanguageServer").get("github_name");
    // get workspace folder
    let workspaceFolders = vscode.workspace.workspaceFolders;
    if (!workspaceFolders) {
        vscode.window.showErrorMessage("No workspace are opened.");
        return 1;
    }
    //Select the rootpath
    let rootpath = workspaceFolders === null || workspaceFolders === void 0 ? void 0 : workspaceFolders[0].uri;
    let rootname = workspaceFolders === null || workspaceFolders === void 0 ? void 0 : workspaceFolders[0].name;
    // create a scripting folder if it doesn't exist
    let scriptingFolderPath = path.join(rootpath.fsPath, "scripting");
    if (!fs.existsSync(scriptingFolderPath)) {
        fs.mkdirSync(scriptingFolderPath);
    }
    // Check if file already exists
    let scriptFileName = rootname + ".sp";
    let scriptFilePath = path.join(rootpath.fsPath, "scripting", scriptFileName);
    if (fs.existsSync(scriptFilePath)) {
        vscode.window.showErrorMessage(scriptFileName + " already exists, aborting.");
        return 1;
    }
    let myExtDir = vscode.extensions.getExtension("Sarrus.sourcepawn-vscode").extensionPath;
    let tasksTemplatesPath = path.join(myExtDir, "templates/plugin_template.sp");
    fs.copyFileSync(tasksTemplatesPath, scriptFilePath);
    // Replace placeholders
    try {
        let data = fs.readFileSync(scriptFilePath, 'utf8');
        let result = data.replace(/\${author_name}/gm, author_name);
        result = result.replace(/\${plugin_name}/gm, rootname);
        result = result.replace(/\${github_name}/gm, github_name);
        fs.writeFileSync(scriptFilePath, result, 'utf8');
    }
    catch (err) {
        console.log(err);
        return 1;
    }
    return 0;
}
exports.run = run;
//# sourceMappingURL=createScript.js.map