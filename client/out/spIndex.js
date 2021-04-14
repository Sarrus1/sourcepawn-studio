"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = void 0;
const vscode_1 = require("vscode");
const spLinter_1 = require("./spLinter");
const glob = require("glob");
const spMode_1 = require("./spMode");
const spProviders_1 = require("./Providers/spProviders");
const registerCommands_1 = require("./Commands/registerCommands");
const spFormat_1 = require("./spFormat");
const path_1 = require("path");
const vscode_uri_1 = require("vscode-uri");
let getDirectories = function (src, ext, callback) {
    glob(src + '/**/*.' + ext, callback);
};
function activate(context) {
    let providers = new spProviders_1.Providers(context.globalState);
    let formatter = new spFormat_1.SMDocumentFormattingEditProvider();
    // Parse files at document opening.
    let sm_home = vscode_1.workspace.getConfiguration("sourcepawnLanguageServer").get("sourcemod_home");
    providers.parse_sm_api(sm_home);
    let workspace = vscode_1.workspace.workspaceFolders[0];
    if (typeof workspace != "undefined") {
        getDirectories(workspace.uri.fsPath, "sp", function (err, res) {
            if (err) {
                console.log("Couldn't read .sp file, ignoring : ", err);
            }
            else {
                for (let file of res) {
                    providers.handle_document_opening(file);
                    providers.completionsProvider.documents.set(path_1.basename(file), vscode_uri_1.URI.file(file));
                }
            }
        });
        getDirectories(workspace.uri.fsPath, "inc", function (err, res) {
            if (err) {
                console.log("Couldn't read .inc file, ignoring : ", err);
            }
            else {
                for (let file of res) {
                    providers.completionsProvider.documents.set(path_1.basename(file), vscode_uri_1.URI.file(file));
                }
            }
        });
    }
    context.subscriptions.push(providers.completionsProvider);
    context.subscriptions.push(vscode_1.languages.registerCompletionItemProvider(spMode_1.SM_MODE, providers.completionsProvider));
    context.subscriptions.push(vscode_1.languages.registerSignatureHelpProvider(spMode_1.SM_MODE, providers.completionsProvider, "("));
    context.subscriptions.push(vscode_1.languages.registerDefinitionProvider(spMode_1.SM_MODE, providers.definitionsProvider));
    context.subscriptions.push(vscode_1.languages.registerDocumentFormattingEditProvider(spMode_1.SM_MODE, formatter));
    context.subscriptions.push(vscode_1.languages.registerHoverProvider(spMode_1.SM_MODE, providers.hoverProvider));
    // Passing providers as an arguments is required to be able to use 'this' in the callbacks.
    vscode_1.workspace.onDidChangeTextDocument(providers.handle_document_change, providers, context.subscriptions);
    vscode_1.workspace.onDidOpenTextDocument(providers.handle_new_document, providers, context.subscriptions);
    vscode_1.workspace.onDidCreateFiles(providers.handle_added_document, providers, context.subscriptions);
    // Register SM Commands
    registerCommands_1.registerSMCommands(context);
    // Register SM linter
    spLinter_1.registerSMLinter(context);
}
exports.activate = activate;
//# sourceMappingURL=spIndex.js.map