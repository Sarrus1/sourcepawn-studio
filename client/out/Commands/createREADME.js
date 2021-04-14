"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.run = void 0;
const vscode = require("vscode");
const fs = require("fs");
const path = require("path");
function run(args) {
    let github_name = vscode.workspace.getConfiguration("sourcepawnLanguageServer").get("github_name");
    if (!github_name) {
        vscode.window
            .showWarningMessage("You didn't specify a GitHub username.", "Open Settings")
            .then((choice) => {
            if (choice === "Open Settings") {
                vscode.commands.executeCommand("workbench.action.openWorkspaceSettings");
            }
        });
    }
    // get workspace folder
    let workspaceFolders = vscode.workspace.workspaceFolders;
    if (!workspaceFolders) {
        vscode.window.showErrorMessage("No workspace are opened.");
        return 1;
    }
    //Select the rootpath
    let rootpath = workspaceFolders === null || workspaceFolders === void 0 ? void 0 : workspaceFolders[0].uri;
    let rootname = workspaceFolders === null || workspaceFolders === void 0 ? void 0 : workspaceFolders[0].name;
    // Check if README.md already exists
    let readmeFilePath = path.join(rootpath.fsPath, "README.md");
    if (fs.existsSync(readmeFilePath)) {
        vscode.window.showErrorMessage("README.md already exists, aborting.");
        return 1;
    }
    let myExtDir = vscode.extensions.getExtension("Sarrus.sourcepawn-vscode").extensionPath;
    let tasksTemplatesPath = path.join(myExtDir, "templates/README_template.MD");
    fs.copyFileSync(tasksTemplatesPath, readmeFilePath);
    // Replace placeholders
    try {
        let result = fs.readFileSync(readmeFilePath, 'utf8');
        result = result.replace(/\${plugin_name}/gm, rootname);
        result = result.replace(/\${github_name}/gm, github_name);
        fs.writeFileSync(readmeFilePath, result, 'utf8');
    }
    catch (err) {
        console.log(err);
        return 1;
    }
    return 0;
}
exports.run = run;
//# sourceMappingURL=createREADME.js.map