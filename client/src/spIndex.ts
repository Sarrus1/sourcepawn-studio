import {ExtensionContext, workspace as Workspace, WorkspaceFolder, languages} from "vscode";
import { registerSMLinter } from "./spLinter";
import * as glob from "glob";
import { SM_MODE } from "./spMode";
import { Providers } from "./Providers/spProviders";
import { registerSMCommands } from "./Commands/registerCommands"; 
import { SMDocumentFormattingEditProvider } from "./spFormat";
import {basename} from "path";
import {URI } from "vscode-uri";


let getDirectories = function (src, ext, callback) {
  glob(src + '/**/*.' + ext, callback);
};


export function activate(context: ExtensionContext) {
  let providers = new Providers(context.globalState);
  let formatter = new SMDocumentFormattingEditProvider();
  // Parse files at document opening.
  let sm_home : string = Workspace.getConfiguration("sourcepawn").get(
		"sourcemod_home");
  providers.parse_sm_api(sm_home);
  let workspace : WorkspaceFolder = Workspace.workspaceFolders[0];
  if(typeof workspace != "undefined")
  {
    getDirectories(workspace.uri.fsPath, "sp", function (err, res) {
      if (err) {
        console.log("Couldn't read .sp file, ignoring : ", err);
      } else {
        for(let file of res)
        {
          providers.handle_document_opening(file);
          providers.completionsProvider.documents.set(basename(file), URI.file(file));
        }
      }
    });
    getDirectories(workspace.uri.fsPath, "inc", function (err, res) {
      if (err) {
        console.log("Couldn't read .inc file, ignoring : ", err);
      } else {
        for(let file of res)
        {
          providers.completionsProvider.documents.set(basename(file), URI.file(file));
        }
      }
    });
  }

  context.subscriptions.push(providers.completionsProvider);
	context.subscriptions.push(languages.registerCompletionItemProvider(SM_MODE , providers.completionsProvider));
	context.subscriptions.push(languages.registerSignatureHelpProvider(SM_MODE, providers.completionsProvider, "("));
  context.subscriptions.push(languages.registerDefinitionProvider(SM_MODE, providers.definitionsProvider));
  context.subscriptions.push(languages.registerDocumentFormattingEditProvider(SM_MODE, formatter));
	context.subscriptions.push(languages.registerHoverProvider(SM_MODE, providers.hoverProvider));
  // Passing providers as an arguments is required to be able to use 'this' in the callbacks.
	Workspace.onDidChangeTextDocument(providers.handle_document_change, providers, context.subscriptions);
	Workspace.onDidOpenTextDocument(providers.handle_new_document, providers, context.subscriptions);
  Workspace.onDidCreateFiles(providers.handle_added_document, providers, context.subscriptions);

  // Register SM Commands
  registerSMCommands(context);

	// Register SM linter
  registerSMLinter(context);
}
