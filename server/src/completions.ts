import { 
    CompletionItemKind, CompletionItem, TextDocumentPositionParams, SignatureHelp, SignatureInformation, TextDocuments,
    TextDocumentChangeEvent, Files
} from 'vscode-languageserver';

import { parse_blob, parse_file } from './parser';

import * as glob from 'glob';
import * as path from 'path';
import Uri from 'vscode-uri';
import * as fs from 'fs';

export interface Completion {
    name: string;
    kind: CompletionItemKind;
    description?: string;

    to_completion_item(): CompletionItem;
    get_signature(): SignatureInformation;
}

export type FunctionParam = {
    label: string,
    documentation: string
}

export class FunctionCompletion implements Completion {
    name: string;
    description: string;
    detail: string;
    params: FunctionParam[];
    kind = CompletionItemKind.Function;

    constructor(name: string, detail: string, description: string, params: FunctionParam[]) {
        this.description = description;
        this.name = name;
        this.params = params;
        this.detail = detail;
    }

    to_completion_item(): CompletionItem {
        return {
            label: this.name,
            kind: this.kind,
            detail: this.description,
        };
    }

    get_signature(): SignatureInformation {
        return {
            label: this.detail,
            documentation: this.description,
            parameters: this.params
        };
    }
}

export class MethodCompletion implements Completion {
    name: string;
    method_map: string;
    description: string;
    detail: string;
    params: FunctionParam[];
    kind = CompletionItemKind.Method;
    
    constructor(method_map: string, name: string, detail: string, description: string, params: FunctionParam[]) {
        this.method_map = method_map;
        this.name = name;
        this.detail = detail;
        this.description = description;
        this.params = params;
    }

    to_completion_item(): CompletionItem {
        return {
            label: `${this.method_map}.${this.name}`,
            insertText: this.name,
            filterText: this.name,
            kind: this.kind,
            detail: this.description
        };
    }

    get_signature(): SignatureInformation {
        return {
            label: this.detail,
            documentation: this.description,
            parameters: this.params
        };
    }
}

export class DefineCompletion implements Completion {
    name: string;
    kind = CompletionItemKind.Variable;

    constructor(name: string) {
        this.name = name;
    }

    to_completion_item(): CompletionItem {
        return {
            label: this.name,
            kind: this.kind,
        };
    }

    get_signature(): SignatureInformation {
        return undefined;
    }
}

export class FileCompletions {
    completions: Map<string, Completion>;
    includes: string[]
    uri: string;

    constructor(uri: string) {
        this.completions = new Map();
        this.includes = [];
        this.uri = uri;
    }

    add(id: string, completion: Completion) {
        this.completions.set(id, completion);
    }

    get(id: string): Completion {
        return this.completions.get(id);
    }

    get_completions(repo: CompletionRepository): Completion[] {
        let completions = [];
        for (let completion of this.completions.values()) {
            completions.push(completion);
        }

        for (let file of this.includes) {
            completions = completions.concat(repo.get_file_completions(file))
        }

        return completions;
    }

    add_include(include: string) {
        this.includes.push(include);
    }

    resolve_import(file: string, relative: boolean = false) {
        let uri = file + ".inc";
        if (!relative) {
            uri = "file://__sourcemod_builtin/" + uri;
            this.add_include(uri);
        } else {
            let base_file = Files.uriToFilePath(this.uri);
            let base_directory = path.dirname(base_file);

            let inc_file = path.resolve(base_directory, uri);
            if (fs.existsSync(inc_file)) {
                uri = Uri.file(inc_file).toString();
                this.add_include(uri);
            } else {
                uri = Uri.file(path.resolve(file + ".sp")).toString();
                this.add_include(uri);
            }
        }
    }
}

export class CompletionRepository {
    completions: Map<string, FileCompletions>;
    documents: TextDocuments;

    constructor(documents: TextDocuments) {
        this.completions = new Map();
        this.documents = documents;

        documents.onDidOpen(this.handle_document_change.bind(this));
        documents.onDidChangeContent(this.handle_document_change.bind(this));
    }

    handle_document_change(event: TextDocumentChangeEvent) {
        let completions = new FileCompletions(event.document.uri);
        parse_blob(event.document.getText(), completions);

        this.read_unscanned_imports(completions);

        this.completions.set(event.document.uri, completions);
    }

    read_unscanned_imports(completions: FileCompletions) {
        for (let import_file of completions.includes) {
            let completion = this.completions.get(import_file);
            if (!completion) {
                let file = Files.uriToFilePath(import_file);
                let new_completions = new FileCompletions(import_file);
                parse_file(file, new_completions);

                this.read_unscanned_imports(new_completions);

                this.completions.set(import_file, new_completions);
            }
        }
    }

    parse_sm_api(sourcemod_home: string) {
        glob(path.join(sourcemod_home, '**/*.inc'), (err, files) => {
            for (let file of files) {
                let completions = new FileCompletions(Uri.file(file).toString());
                parse_file(file, completions);

                let uri = "file://__sourcemod_builtin/" + path.relative(sourcemod_home, file);
                this.completions.set(uri, completions);
            }
        });
    }

    get_completions(position: TextDocumentPositionParams): CompletionItem[] {
        let document = this.documents.get(position.textDocument.uri);
        let is_method = false;
        if (document) {
            let line = document.getText().split("\n")[position.position.line].trim();
            for (let i = line.length - 2; i >= 0; i--) {
                if (line[i].match(/[a-zA-Z0-9_]/)) {
                    continue;
                }

                if (line[i] === '.') {
                    is_method = true;
                    break;
                }

                break;
            }
        }

        let all_completions = this.get_file_completions(position.textDocument.uri).map((completion) => completion.to_completion_item());
    
        if (is_method) {
            return all_completions.filter(completion => completion.kind === CompletionItemKind.Method);
        } else {
            return all_completions.filter(completion => completion.kind !== CompletionItemKind.Method);
        }
    }

    get_file_completions(file: string): Completion[] {
        let completions = this.completions.get(file);
        if (completions) {
            return completions.get_completions(this);
        }
        
        return [];
    }

    get_signature(position: TextDocumentPositionParams): SignatureHelp {
        let document = this.documents.get(position.textDocument.uri);
        if (document) {
            let {method, parameter_count} = (() => {
                let line = document.getText().split("\n")[position.position.line];

                if (line[position.position.character - 1] === ")") {
                    // We've finished this call
                    return {method: undefined, parameter_count: 0};
                }

                let method = "";
                let end_parameters = false;
                let parameter_count = 0;

                for (let i = position.position.character; i >= 0; i--) {
                    if (end_parameters) {
                        if (line[i].match(/[A-Za-z0-9_]/)) {
                            method = line[i] + method;
                        } else {
                            break;
                        }
                    } else {
                        if (line[i] === "(") {
                            end_parameters = true;
                        } else if (line[i] === ",") {
                            parameter_count++;
                        }
                    }
                }

                return {method, parameter_count};
            })();

            let completions = this.get_file_completions(position.textDocument.uri).filter((completion) => {
                return completion.name === method;
            });

            if (completions.length > 0) {
                return {
                    signatures: [ completions[0].get_signature() ],
                    activeParameter: parameter_count,
                    activeSignature: 0
                };
            }
        }
        
        return {
            signatures: [ ],
            activeSignature: 0,
            activeParameter: 0
        };
    }
}