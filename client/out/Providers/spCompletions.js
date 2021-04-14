"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.CompletionRepository = exports.FileCompletions = void 0;
const vscode = require("vscode");
const path = require("path");
const fs_1 = require("fs");
const vscode_uri_1 = require("vscode-uri");
const spCompletionsKinds_1 = require("./spCompletionsKinds");
class FileCompletions {
    constructor(uri) {
        this.completions = new Map();
        this.includes = [];
        this.uri = uri;
    }
    add(id, completion) {
        this.completions.set(id, completion);
    }
    get(id) {
        return this.completions.get(id);
    }
    get_completions(repo) {
        let completions = [];
        for (let completion of this.completions.values()) {
            completions.push(completion);
        }
        return completions;
    }
    to_completion_resolve(item) {
        item.label = item.label;
        item.documentation = item.documentation;
        return item;
    }
    add_include(include, IsBuiltIn) {
        this.includes.push(new spCompletionsKinds_1.Include(include, IsBuiltIn));
    }
    resolve_import(file, documents, IsBuiltIn = false) {
        let inc_file;
        // If no extension is provided, it's a .inc file
        if (!/.sp\s*$/g.test(file)) {
            file += ".inc";
        }
        let match = file.match(/[A-z0-9_.]*$/);
        if (match)
            file = match[0];
        let uri;
        if (!(uri = documents.get(file))) {
            let includes_dirs = vscode.workspace
                .getConfiguration("sourcepawnLanguageServer")
                .get("optionalIncludeDirsPaths");
            for (let includes_dir of includes_dirs) {
                inc_file = path.join(includes_dir, file);
                if (fs_1.existsSync(inc_file)) {
                    this.add_include(vscode_uri_1.URI.file(inc_file).toString(), IsBuiltIn);
                    return;
                }
            }
            this.add_include("file://__sourcemod_builtin/" + file, IsBuiltIn);
        }
        else {
            this.add_include(uri.toString(), IsBuiltIn);
        }
    }
}
exports.FileCompletions = FileCompletions;
class CompletionRepository {
    constructor(globalState) {
        this.completions = new Map();
        this.documents = new Map();
        this.globalState = globalState;
    }
    provideCompletionItems(document, position, token) {
        let completions = this.get_completions(document, position);
        return completions;
    }
    dispose() { }
    get_completions(document, position) {
        let is_method = false;
        if (document) {
            let line = document.getText().split("\n")[position.line].trim();
            for (let i = line.length - 2; i >= 0; i--) {
                if (line[i].match(/[a-zA-Z0-9_]/)) {
                    continue;
                }
                if (line[i] === ".") {
                    is_method = true;
                    break;
                }
                break;
            }
        }
        let all_completions = this.get_all_completions(document.uri.toString());
        let all_completions_list = new vscode.CompletionList();
        if (all_completions != []) {
            all_completions_list.items = all_completions.map((completion) => {
                if (completion) {
                    if (completion.to_completion_item) {
                        return completion.to_completion_item(document.uri.fsPath);
                    }
                }
            });
        }
        //return all_completions_list;
        if (is_method) {
            all_completions_list.items.filter((completion) => completion.kind === vscode.CompletionItemKind.Method);
            return all_completions_list;
        }
        else {
            all_completions_list.items.filter((completion) => completion.kind !== vscode.CompletionItemKind.Method);
            return all_completions_list;
        }
    }
    get_all_completions(file) {
        let completion = this.completions.get(file);
        let includes = new Set();
        if (completion) {
            this.get_included_files(completion, includes);
        }
        includes.add(file);
        return [...includes]
            .map((file) => {
            return this.get_file_completions(file);
        })
            .reduce((completion, file_completions) => completion.concat(file_completions), []);
    }
    get_file_completions(file) {
        let file_completions = this.completions.get(file);
        let completion_list = [];
        if (file_completions) {
            return file_completions.get_completions(this);
        }
        return completion_list;
    }
    get_included_files(completions, files) {
        for (let include of completions.includes) {
            if (!files.has(include.uri)) {
                files.add(include.uri);
                let include_completions = this.completions.get(include.uri);
                if (include_completions) {
                    this.get_included_files(include_completions, files);
                }
            }
        }
    }
    provideHover(document, position, token) {
        let range = document.getWordRangeAtPosition(position);
        let word = document.getText(range);
        let completions = this.get_all_completions(document.uri.toString()).filter((completion) => {
            return completion.name === word;
        });
        if (completions.length > 0) {
            return completions[0].get_hover();
        }
    }
    provideSignatureHelp(document, position, token) {
        if (document) {
            let { method, parameter_count } = (() => {
                let line = document.getText().split("\n")[position.line];
                if (line[position.character - 1] === ")") {
                    // We've finished this call
                    return { method: undefined, parameter_count: 0 };
                }
                let method = "";
                let end_parameters = false;
                let parameter_count = 0;
                for (let i = position.character; i >= 0; i--) {
                    if (end_parameters) {
                        if (line[i].match(/[A-Za-z0-9_]/)) {
                            method = line[i] + method;
                        }
                        else {
                            break;
                        }
                    }
                    else {
                        if (line[i] === "(") {
                            end_parameters = true;
                        }
                        else if (line[i] === ",") {
                            parameter_count++;
                        }
                    }
                }
                return { method, parameter_count };
            })();
            let completions = this.get_all_completions(document.uri.toString()).filter((completion) => {
                return completion.name === method;
            });
            if (completions.length > 0) {
                return {
                    signatures: [completions[0].get_signature()],
                    activeParameter: parameter_count,
                    activeSignature: 0,
                };
            }
        }
        return {
            signatures: [],
            activeSignature: 0,
            activeParameter: 0,
        };
    }
}
exports.CompletionRepository = CompletionRepository;
//# sourceMappingURL=spCompletions.js.map