"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.run = void 0;
const vscode = require("vscode");
const fs = require("fs");
const path = require("path");
const CreateTaskCommand = require("./createTask");
const CreateScriptCommand = require("./createScript");
const CreateREADMECommand = require("./createREADME");
const CreateMasterCommand = require("./createGitHubActions");
async function run(args) {
    // get workspace folder
    let workspaceFolders = vscode.workspace.workspaceFolders;
    if (!workspaceFolders) {
        vscode.window.showErrorMessage("No workspace are opened.");
        return;
    }
    //Select the rootpath
    let rootpath = workspaceFolders === null || workspaceFolders === void 0 ? void 0 : workspaceFolders[0].uri;
    let rootname = workspaceFolders === null || workspaceFolders === void 0 ? void 0 : workspaceFolders[0].name;
    // Create the plugins folder
    let pluginsFolderPath = path.join(rootpath.fsPath, "plugins");
    if (!fs.existsSync(pluginsFolderPath)) {
        fs.mkdirSync(pluginsFolderPath);
    }
    // Running the other commands
    CreateTaskCommand.run(undefined);
    CreateScriptCommand.run(undefined);
    CreateREADMECommand.run(undefined);
    CreateMasterCommand.run(undefined);
}
exports.run = run;
//# sourceMappingURL=createProject.js.map