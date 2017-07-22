import { CompletionItemKind, CompletionItem, TextDocumentPositionParams, SignatureHelp, SignatureInformation, TextDocuments } from 'vscode-languageserver';

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

export class CompletionRepository {
    completions: Map<string, Completion>;
    documents: TextDocuments;

    constructor(documents: TextDocuments) {
        this.completions = new Map();
        this.documents = documents;
    }

    add(id: string, completion: Completion) {
        this.completions.set(id, completion);
    }

    get(id: string): Completion {
        return this.completions.get(id);
    }

    get_completions(position: TextDocumentPositionParams): CompletionItem[] {
        // TODO: Filter completions
        let completions = [];
        for (let completion of this.completions.values()) {
            completions.push(completion.to_completion_item());
        }

        return completions;
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

            let completion = this.get(method);
            if (completion) {
                return {
                    signatures: [ completion.get_signature() ],
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