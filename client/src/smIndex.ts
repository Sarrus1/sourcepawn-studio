import * as vscode from "vscode";
import * as CreateTaskCommand from "./commands/createTask";
import * as CreateScriptCommand from "./commands/createScript";
import * as CreateREADMECommand from "./commands/createREADME";
import * as CreateMasterCommand from "./commands/createGitHubActions";
import * as CreateProjectCommand from "./commands/createProject";
import * as CompileSMCommand from "./commands/compileSM";
import * as linter from "./smLinter";
import { SM_MODE } from "./smMode";
import { Providers } from "./smProviders";


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
  

	// Register commands
  let createTask = vscode.commands.registerCommand(
    "extension.createTask",
    CreateTaskCommand.run.bind(undefined)
  );
  context.subscriptions.push(createTask);

  let createScript = vscode.commands.registerCommand(
    "extension.createScript",
    CreateScriptCommand.run.bind(undefined)
  );
  context.subscriptions.push(createScript);

  let createREADME = vscode.commands.registerCommand(
    "extension.createREADME",
    CreateREADMECommand.run.bind(undefined)
  );
  context.subscriptions.push(createREADME);

  let createMaster = vscode.commands.registerCommand(
    "extension.createMaster",
    CreateMasterCommand.run.bind(undefined)
  );
  context.subscriptions.push(createMaster);

  let createProject = vscode.commands.registerCommand(
    "extension.createProject",
    CreateProjectCommand.run.bind(undefined)
  );
  context.subscriptions.push(createProject);

	let compileSM = vscode.commands.registerCommand(
    "extension.compileSM",
    CompileSMCommand.run.bind(undefined)
  );
  context.subscriptions.push(compileSM);

	// Register linter
  context.subscriptions.push(linter.compilerDiagnostics);
  context.subscriptions.push(linter.activeEditorChanged);
  context.subscriptions.push(linter.textDocumentChanged);
  context.subscriptions.push(linter.textDocumentClosed);
}
