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
    workspaceRoot = params.workspaceFolders[0].uri;
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


function init_parse_sm_api(sm_home: string) {
    if (sm_home) {
        completions.parse_sm_api(sm_home);
    }
}

connection.onDidChangeConfiguration((change) => {
    let sm_home = change.settings.sourcepawnLanguageServer.sourcemod_home;
    init_parse_sm_api(sm_home);
})

connection.onCompletion((textDocumentPosition) => {
    return completions.get_completions(textDocumentPosition);
});

connection.onSignatureHelp((textDocumentPosition) => {
    return completions.get_signature(textDocumentPosition);
});

connection.listen();