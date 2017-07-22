import { 
    IPCMessageReader, IPCMessageWriter ,IConnection, createConnection,
    TextDocuments, CompletionItemKind, CompletionItem, TextDocumentSyncKind
} from "vscode-languageserver";

import * as glob from 'glob';
import * as path from 'path';

import { Completion, CompletionRepository } from './completions';
import { parse_file } from './parser';

let connection = createConnection(new IPCMessageReader(process), new IPCMessageWriter(process));
let documents = new TextDocuments();
documents.listen(connection);

let completions = new CompletionRepository(documents);

let workspaceRoot: string;

connection.onInitialize((params) => {
    workspaceRoot = params.rootPath;

    return {
        capabilities: {
            textDocumentSync: documents.syncKind,
            completionProvider: {
                resolveProvider: false
            },
            signatureHelpProvider: {
                triggerCharacters: ["("]
            }
        }
    };
});

connection.onDidChangeConfiguration((change) => {
    let sm_home = change.settings.sourcepawnLanguageServer.sourcemod_home;
    if (sm_home) {
        completions.parse_sm_api(sm_home);
    }
})

connection.onCompletion((textDocumentPosition) => {
    return completions.get_completions(textDocumentPosition);
});

connection.onSignatureHelp((textDocumentPosition) => {
    return completions.get_signature(textDocumentPosition);
});

connection.listen();