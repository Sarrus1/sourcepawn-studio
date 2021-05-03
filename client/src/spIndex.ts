import {
  ExtensionContext,
  workspace as Workspace,
  WorkspaceFolder,
  languages,
	window,
} from "vscode";
import { registerSMLinter } from "./spLinter";
import * as glob from "glob";
import { SP_MODE } from "./spMode";
import { Providers } from "./Providers/spProviders";
import { registerSMCommands } from "./Commands/registerCommands";
import { SMDocumentFormattingEditProvider } from "./spFormat";
import { basename, extname } from "path";
import { URI } from "vscode-uri";
import { type } from "os";

let getDirectories = function (src, ext, callback) {
  glob(src + "/**/*", callback);
};

export function activate(context: ExtensionContext) {
  const providers = new Providers(context.globalState);
  let formatter = new SMDocumentFormattingEditProvider();
  // Parse files at document opening.
  let sm_home: string = Workspace.getConfiguration("sourcepawn").get(
    "SourcemodHome"
  );
  providers.parse_sm_api(sm_home);
  let workspaceFolders = Workspace.workspaceFolders;
	if(typeof workspaceFolders == "undefined")
	{
		window.showErrorMessage(
			"No workspace or folder found. \n Please open the folder containing your .sp file, not just the .sp file."
		);
		return;
	}
	let workspace = workspaceFolders[0];
  if (typeof workspace != "undefined") {
    getDirectories(workspace.uri.fsPath, "sp", function (err, res) {
      if (err) {
        console.log("Couldn't read .sp file, ignoring : ", err);
      } else {
        for (let file of res) {
					let FileExt:string = extname(file);
					if(FileExt == ".sp")
					{
						providers.handle_document_opening(file);
					}
					if(FileExt == ".sp" || FileExt == ".inc")
					{
						providers.completionsProvider.documents.set(
							basename(file),
							URI.file(file)
						);
					}
        }
      }
    });
  }

  context.subscriptions.push(providers.completionsProvider);
  context.subscriptions.push(
    languages.registerCompletionItemProvider(
      SP_MODE,
      providers.completionsProvider
    )
  );
  context.subscriptions.push(
    languages.registerCompletionItemProvider(
      SP_MODE,
      providers.documentationProvider,
      "*"
    )
  );
  context.subscriptions.push(
    languages.registerSignatureHelpProvider(
      SP_MODE,
      providers.completionsProvider,
      "("
    )
  );
  context.subscriptions.push(
    languages.registerDefinitionProvider(SP_MODE, providers.definitionsProvider)
  );

  context.subscriptions.push(
    languages.registerDocumentFormattingEditProvider(SP_MODE, formatter)
  );
  context.subscriptions.push(
    languages.registerHoverProvider(SP_MODE, providers.hoverProvider)
  );
  // Passing providers as an arguments is required to be able to use 'this' in the callbacks.
  Workspace.onDidChangeTextDocument(
    providers.handle_document_change,
    providers,
    context.subscriptions
  );
  Workspace.onDidOpenTextDocument(
    providers.handle_new_document,
    providers,
    context.subscriptions
  );
  Workspace.onDidCreateFiles(
    providers.handle_added_document,
    providers,
    context.subscriptions
  );

  // Register SM Commands
  registerSMCommands(context);

  // Register SM linter
  registerSMLinter(context);
}
