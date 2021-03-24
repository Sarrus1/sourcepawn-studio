import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind
} from "vscode-languageclient/node";
import * as glob from "glob";
import * as path from "path";
import * as CreateTaskCommand from "./commands/createTask";
import * as CreateScriptCommand from "./commands/createScript";
import * as CreateREADMECommand from "./commands/createREADME";
import * as CreateMasterCommand from "./commands/createGitHubActions";
import * as CreateProjectCommand from "./commands/createProject";
import * as CompileSMCommand from "./commands/compileSM";
import * as linter from "./smLinter";
import {SM_MODE} from "./smMode";
import { CompletionRepository } from "./smCompletions"



export function activate(context: vscode.ExtensionContext) {
  let clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "sourcepawn" }],
    synchronize: {
      //configurationSection: 'sourcepawnLanguageServer',
      fileEvents: [
        vscode.workspace.createFileSystemWatcher("**/*.sp"),
        vscode.workspace.createFileSystemWatcher("**/*.inc"),
      ],
    },
  };
	let completions = new CompletionRepository(context.globalState);

	let sm_home : string = vscode.workspace.getConfiguration("sourcepawnLanguageServer").get(
		"sourcemod_home");
	completions.parse_sm_api(sm_home);
	context.subscriptions.push(completions);
	context.subscriptions.push(vscode.languages.registerCompletionItemProvider(SM_MODE ,completions));
	context.subscriptions.push(vscode.languages.registerSignatureHelpProvider(SM_MODE, completions, "("));
	vscode.workspace.onDidChangeTextDocument(completions.handle_document_change, completions, context.subscriptions);
	vscode.workspace.onDidOpenTextDocument(completions.handle_new_document, null, context.subscriptions);
  
	
	
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
