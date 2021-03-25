import * as vscode from "vscode";
import { registerSMLinter } from "./smLinter";
import { SM_MODE } from "./smMode";
import { Providers } from "./Providers/smProviders";
import { registerSMCommands } from "./Commands/registerCommands"; 


export function activate(context: vscode.ExtensionContext) {
  let providers = new Providers(context.globalState);
  
	let sm_home : string = vscode.workspace.getConfiguration("sourcepawnLanguageServer").get(
		"sourcemod_home");
  providers.parse_sm_api(sm_home);

  context.subscriptions.push(providers.completionsProvider);
	context.subscriptions.push(vscode.languages.registerCompletionItemProvider(SM_MODE , providers.completionsProvider));
	context.subscriptions.push(vscode.languages.registerSignatureHelpProvider(SM_MODE, providers.completionsProvider, "("));
  context.subscriptions.push(vscode.languages.registerDefinitionProvider(SM_MODE, providers.definitionsProvider));

  // Passing providers as an arguments is required to be able to use 'this' in the callbacks.
	vscode.workspace.onDidChangeTextDocument(providers.handle_document_change, providers, context.subscriptions);
	vscode.workspace.onDidOpenTextDocument(providers.handle_new_document, providers, context.subscriptions);
  
  // Register SM Commands
  registerSMCommands(context);

	// Register SM linter
  registerSMLinter(context);
}
