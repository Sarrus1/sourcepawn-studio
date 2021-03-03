import {
	TextDocument
} from 'vscode-languageserver-textdocument';
import { 
    createConnection,
    TextDocuments,
    TextDocumentSyncKind,
    ProposedFeatures,
		DidChangeConfigurationNotification,
		InitializeParams
} from "vscode-languageserver/node";

import { CompletionRepository } from './completions';

//let sm_home: string = Workspace.getConfiguration("sourcepawnLanguageServer").get("sourcemod_home");
let connection = createConnection(ProposedFeatures.all);
let documents: TextDocuments<TextDocument> = new TextDocuments(TextDocument);
documents.listen(connection);

let completions = new CompletionRepository(documents);

let workspaceRoot: string;

let hasConfigurationCapability: boolean = false;

connection.onInitialize((params) => {
    workspaceRoot = params.workspaceFolders[0].uri;
		let capabilities = params.capabilities;
		hasConfigurationCapability = !!(
			capabilities.workspace && !!capabilities.workspace.configuration
		);
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

connection.onInitialized(async (params: InitializeParams) => {
	if (hasConfigurationCapability) {
    // Register for all configuration changes.
    connection.client.register(DidChangeConfigurationNotification.type, undefined);
	}
	let sm_home = await f1();
	init_parse_sm_api(sm_home);
});

async function f1() {
  let config = (await getDocumentSettings(workspaceRoot)).sourcemod_home;
	return config;
}

function rejected(result) {
  console.error(result);
}

async function init_parse_sm_api(sm_home) {
	console.debug("DEBUG1", sm_home);
	if (sm_home) {
		console.debug("DEBUG2", sm_home);
		completions.parse_sm_api(sm_home);
	}
}

interface SourcepawnSettings {
  sourcemod_home: string,
	ServerOutput: string
}

// The global settings, used when the `workspace/configuration` request is not supported by the client.
// Please note that this is not the case when using this server with the client provided in this example
// but could happen with other clients.
const defaultSettings: SourcepawnSettings = { sourcemod_home: "",
																							ServerOutput: "off" };
let globalSettings: SourcepawnSettings = defaultSettings;

// Cache the settings of all open documents
let documentSettings: Map<string, Thenable<SourcepawnSettings>> = new Map();

connection.onDidChangeConfiguration((change) => {
	let sm_home = change.settings.sourcepawnLanguageServer.sourcemod_home;
	if(sm_home) {
		completions.parse_sm_api(sm_home);
	}
})
// Qu'est ce qu'on passe à la fonction en fait?
function getDocumentSettings(resource: string): Thenable<SourcepawnSettings> {
  if (!hasConfigurationCapability) {
    return Promise.resolve(globalSettings);
  }
  let result = documentSettings.get(resource);
  if (!result) {
    result = connection.workspace.getConfiguration({
      scopeUri: resource,
      section: 'sourcepawnLanguageServer'
    });
    documentSettings.set(resource, result);
  }
  return result;
}

connection.onCompletion((textDocumentPosition) => {
    return completions.get_completions(textDocumentPosition);
});

connection.onSignatureHelp((textDocumentPosition) => {
    return completions.get_signature(textDocumentPosition);
});

connection.listen();