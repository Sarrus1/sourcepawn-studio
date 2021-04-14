"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Providers = void 0;
const glob = require("glob");
const path = require("path");
const vscode_uri_1 = require("vscode-uri");
const fs = require("fs");
const smCompletions = require("./spCompletions");
const smDefinitions = require("./spDefinitions");
const smParser = require("./spParser");
class Providers {
    constructor(globalState) {
        let CompletionRepo = new smCompletions.CompletionRepository(globalState);
        this.completionsProvider = CompletionRepo;
        this.definitionsProvider = new smDefinitions.DefinitionRepository(globalState);
        this.hoverProvider = CompletionRepo;
    }
    handle_added_document(event) {
        for (let file of event.files) {
            this.completionsProvider.documents.set(path.basename(file.fsPath), file);
        }
    }
    handle_document_change(event) {
        let this_completions = new smCompletions.FileCompletions(event.document.uri.toString());
        let file_path = event.document.uri.fsPath;
        this.completionsProvider.documents.set(path.basename(file_path), event.document.uri);
        // Some file paths are appened with .git
        file_path = file_path.replace(".git", "");
        // We use parse_text here, otherwise, if the user didn't save the file, the changes wouldn't be registered.
        try {
            smParser.parse_text(event.document.getText(), file_path, this_completions, this.definitionsProvider.definitions, this.completionsProvider.documents);
        }
        catch (error) {
            console.log(error);
        }
        this.read_unscanned_imports(this_completions);
        this.completionsProvider.completions.set(event.document.uri.toString(), this_completions);
    }
    handle_new_document(document) {
        let this_completions = new smCompletions.FileCompletions(document.uri.toString());
        let file_path = document.uri.fsPath;
        if (path.extname(file_path) == "git")
            return;
        this.completionsProvider.documents.set(path.basename(file_path), document.uri);
        // Some file paths are appened with .git
        //file_path = file_path.replace(".git", "");
        try {
            smParser.parse_file(file_path, this_completions, this.definitionsProvider.definitions, this.completionsProvider.documents);
        }
        catch (error) {
            console.log(error);
        }
        this.read_unscanned_imports(this_completions);
        this.completionsProvider.completions.set(document.uri.toString(), this_completions);
    }
    handle_document_opening(path) {
        let uri = vscode_uri_1.URI.file(path).toString();
        let this_completions = new smCompletions.FileCompletions(uri);
        // Some file paths are appened with .git
        path = path.replace(".git", "");
        try {
            smParser.parse_file(path, this_completions, this.definitionsProvider.definitions, this.completionsProvider.documents);
        }
        catch (error) {
            console.log(error);
        }
        this.read_unscanned_imports(this_completions);
        this.completionsProvider.completions.set(uri, this_completions);
    }
    read_unscanned_imports(completions) {
        for (let import_file of completions.includes) {
            let completion = this.completionsProvider.completions.get(import_file.uri);
            if (typeof completion === "undefined") {
                let file = vscode_uri_1.URI.parse(import_file.uri).fsPath;
                if (fs.existsSync(file)) {
                    let new_completions = new smCompletions.FileCompletions(import_file.uri);
                    smParser.parse_file(file, new_completions, this.definitionsProvider.definitions, this.completionsProvider.documents, import_file.IsBuiltIn);
                    this.read_unscanned_imports(new_completions);
                    this.completionsProvider.completions.set(import_file.uri, new_completions);
                }
            }
        }
    }
    parse_sm_api(sourcemod_home) {
        if (!sourcemod_home)
            return;
        glob(path.join(sourcemod_home, "**/*.inc"), (err, files) => {
            for (let file of files) {
                let completions = new smCompletions.FileCompletions(vscode_uri_1.URI.file(file).toString());
                smParser.parse_file(file, completions, this.definitionsProvider.definitions, this.completionsProvider.documents, true);
                let uri = "file://__sourcemod_builtin/" + path.relative(sourcemod_home, file);
                this.completionsProvider.completions.set(uri, completions);
            }
        });
    }
}
exports.Providers = Providers;
//# sourceMappingURL=spProviders.js.map