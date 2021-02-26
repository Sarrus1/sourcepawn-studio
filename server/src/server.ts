import {
	TextDocument
} from 'vscode-languageserver-textdocument';
import { 
    createConnection,
    TextDocuments,
    TextDocumentSyncKind,
    ProposedFeatures
} from "vscode-languageserver/node";

import { CompletionRepository } from './completions';
let connection = createConnection(ProposedFeatures.all);
let documents: TextDocuments<TextDocument> = new TextDocuments(TextDocument);
documents.listen(connection);

let completions = new CompletionRepository(documents);

let workspaceRoot: string;

connection.onInitialize((params) => {
    workspaceRoot = params.rootUri;

    return {
        capabilities: {
            textDocumentSync: TextDocumentSyncKind.Full,
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