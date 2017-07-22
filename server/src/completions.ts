import { CompletionItemKind, CompletionItem, TextDocumentPositionParams } from 'vscode-languageserver';

export interface Completion {
    name: string;
    kind: CompletionItemKind;
    description?: string;

    to_completion_item(): CompletionItem;
}

export class FunctionCompletion implements Completion {
    name: string;
    description: string;
    kind = CompletionItemKind.Function;
    id: string;

    constructor(id: string, name: string, description: string) {
        this.description = description;
        this.name = name;
        this.id = id;
    }

    to_completion_item(): CompletionItem {
        return {
            label: this.name,
            kind: this.kind,
            documentation: this.description,
            data: this.id
        };
    }
}

export class DefineCompletion implements Completion {
    name: string;
    kind = CompletionItemKind.Variable;
    id: string;

    constructor(id: string, name: string) {
        this.id = id;
        this.name = name;
    }

    to_completion_item(): CompletionItem {
        return {
            label: this.name,
            kind: this.kind
        };
    }
}

export class CompletionRepository {
    completions: Map<string, Completion>;

    constructor() {
        this.completions = new Map();
    }

    add(id: string, completion: Completion) {
        this.completions.set(id, completion);
    }

    get(id: string): Completion {
        return this.completions.get(id);
    }

    get_completions(position: TextDocumentPositionParams): CompletionItem[] {
        let completions = [];
        for (let completion of this.completions.values()) {
            completions.push(completion.to_completion_item());
        }

        return completions;
    }

    resolve_completion(item: CompletionItem) {
        return item;
    }
}