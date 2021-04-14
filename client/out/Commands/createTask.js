"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.run = void 0;
const vscode = require("vscode");
const fs = require("fs");
const path = require("path");
function run(args) {
    // Get configuration
    let sm_home = vscode.workspace.getConfiguration("sourcepawnLanguageServer").get("sourcemod_home");
    if (!sm_home) {
        vscode.window
            .showWarningMessage("SourceMod API not found in the project. You should set SourceMod Home for tasks generation to work.", "Open Settings")
            .then((choice) => {
            if (choice === "Open Settings") {
                vscode.commands.executeCommand("workbench.action.openWorkspaceSettings");
            }
        });
    }
    let spcomp_path = vscode.workspace.getConfiguration("sourcepawnLanguageServer").get("spcomp_path");
    if (!spcomp_path) {
        vscode.window
            .showErrorMessage("SourceMod compiler not found in the project. You need to set spcomp path for tasks generation to work.", "Open Settings")
            .then((choice) => {
            if (choice === "Open Settings") {
                vscode.commands.executeCommand("workbench.action.openWorkspaceSettings");
            }
        });
        return 1;
    }
    // get workspace folder
    let workspaceFolders = vscode.workspace.workspaceFolders;
    if (!workspaceFolders) {
        vscode.window.showErrorMessage("No workspace are opened.");
        return 1;
    }
    //Select the rootpath
    let rootpath = workspaceFolders === null || workspaceFolders === void 0 ? void 0 : workspaceFolders[0].uri;
    // create task folder if it doesn't exist
    let taskFolderPath = path.join(rootpath.fsPath, ".vscode");
    if (!fs.existsSync(taskFolderPath)) {
        fs.mkdirSync(taskFolderPath);
    }
    // Check if file already exists
    let taskFilePath = path.join(rootpath.fsPath, ".vscode/tasks.json");
    if (fs.existsSync(taskFilePath)) {
        vscode.window.showErrorMessage("tasks.json file already exists.");
        return 1;
    }
    let myExtDir = vscode.extensions.getExtension("Sarrus.sourcepawn-vscode").extensionPath;
    let tasksTemplatesPath = path.join(myExtDir, "templates/tasks.json");
    fs.copyFileSync(tasksTemplatesPath, taskFilePath);
    spcomp_path = spcomp_path.replace(/\\/gm, "\\\\");
    sm_home = sm_home.replace(/\\/gm, "\\\\");
    // Replace placeholders
    try {
        let data = fs.readFileSync(taskFilePath, 'utf8');
        let result = data.replace(/\${spcomp_path}/gm, spcomp_path);
        result = result.replace(/\${include_path}/gm, sm_home);
        fs.writeFileSync(taskFilePath, result, 'utf8');
    }
    catch (err) {
        console.log(err);
        return 1;
    }
    return 0;
}
exports.run = run;
//# sourceMappingURL=createTask.js.map