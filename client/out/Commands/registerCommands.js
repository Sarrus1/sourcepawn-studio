"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.registerSMCommands = void 0;
const vscode_1 = require("vscode");
const CreateTaskCommand = require("./createTask");
const CreateScriptCommand = require("./createScript");
const CreateREADMECommand = require("./createREADME");
const CreateMasterCommand = require("./createGitHubActions");
const CreateProjectCommand = require("./createProject");
const CompileSMCommand = require("./compileSM");
function registerSMCommands(context) {
    let createTask = vscode_1.commands.registerCommand("extension.createTask", CreateTaskCommand.run.bind(undefined));
    context.subscriptions.push(createTask);
    let createScript = vscode_1.commands.registerCommand("extension.createScript", CreateScriptCommand.run.bind(undefined));
    context.subscriptions.push(createScript);
    let createREADME = vscode_1.commands.registerCommand("extension.createREADME", CreateREADMECommand.run.bind(undefined));
    context.subscriptions.push(createREADME);
    let createMaster = vscode_1.commands.registerCommand("extension.createMaster", CreateMasterCommand.run.bind(undefined));
    context.subscriptions.push(createMaster);
    let createProject = vscode_1.commands.registerCommand("extension.createProject", CreateProjectCommand.run.bind(undefined));
    context.subscriptions.push(createProject);
    let compileSM = vscode_1.commands.registerCommand("extension.compileSM", CompileSMCommand.run.bind(undefined));
    context.subscriptions.push(compileSM);
}
exports.registerSMCommands = registerSMCommands;
//# sourceMappingURL=registerCommands.js.map