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
    glob(path.join(workspaceRoot, "**/*.inc"), (err, files) => {
        for (let file of files) {
            parse_file(file, completions);
        }
    });

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

connection.onCompletion((textDocumentPosition) => {
    return completions.get_completions(textDocumentPosition);
});

connection.onSignatureHelp((textDocumentPosition) => {
    return completions.get_signature(textDocumentPosition);
});

connection.listen();