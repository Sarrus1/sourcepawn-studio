import * as vscode from "vscode";
import * as glob from "glob";
import * as path from "path";
import { URI } from "vscode-uri";
import * as fs from "fs";
import * as smCompletions from "./smCompletions";
import * as smDefinitions from "./smDefinitions";
import * as smParser from "./smParser";

export class Providers {
  completionsProvider: smCompletions.CompletionRepository;
  definitionsProvider: smDefinitions.DefinitionRepository;

  constructor(globalState?: vscode.Memento) {
    this.completionsProvider = new smCompletions.CompletionRepository(
      globalState
    );
    this.definitionsProvider = new smDefinitions.DefinitionRepository(
      globalState
    );	
  }

	public handle_document_change(event: vscode.TextDocumentChangeEvent) {
    let this_completions : smCompletions.FileCompletions = new smCompletions.FileCompletions(event.document.uri.toString());
		let path : string = event.document.uri.fsPath;
		// Some file paths are appened with .git
		path = path.replace(".git", "");
    // We use parse_text here, otherwise, if the user didn't save the file, the changes wouldn't be registered.
		try{
			smParser.parse_text(event.document.getText(), path, this_completions, this.definitionsProvider.definitions);
		}
		catch(error){console.log(error)}
		this.read_unscanned_imports(this_completions);
    this.completionsProvider.completions.set(event.document.uri.toString(), this_completions);
  }

  public handle_new_document(document: vscode.TextDocument) {
    let this_completions : smCompletions.FileCompletions = new smCompletions.FileCompletions(document.uri.toString());
		let path : string =document.uri.fsPath;
		// Some file paths are appened with .git
		path = path.replace(".git", "");
		try{
			smParser.parse_file(path, this_completions, this.definitionsProvider.definitions);
		}
		catch(error){console.log(error);}

		this.read_unscanned_imports(this_completions);
    this.completionsProvider.completions.set(document.uri.toString(), this_completions);
  }

  public handle_document_opening(path : string)
  {
    let uri : string = URI.file(path).toString();
    let this_completions : smCompletions.FileCompletions = new smCompletions.FileCompletions(uri);
		// Some file paths are appened with .git
		path = path.replace(".git", "");
		try{
			smParser.parse_file(path, this_completions, this.definitionsProvider.definitions);
		}
		catch(error){console.log(error);}

		this.read_unscanned_imports(this_completions);
    this.completionsProvider.completions.set(uri, this_completions);
  }

	public read_unscanned_imports(completions: smCompletions.FileCompletions) {
    for (let import_file of completions.includes) {
      let completion = this.completionsProvider.completions.get(import_file.uri);
      if (typeof completion === "undefined") {
        let file = URI.parse(import_file.uri).fsPath;
        if (fs.existsSync(file)) {
          let new_completions = new smCompletions.FileCompletions(import_file.uri);
          smParser.parse_file(file, new_completions, this.definitionsProvider.definitions, import_file.IsBuiltIn);

          this.read_unscanned_imports(new_completions);

          this.completionsProvider.completions.set(import_file.uri, new_completions);
        }
      }
    }
  }

	public parse_sm_api(sourcemod_home: string): void {
    if (!sourcemod_home) return;
    glob(path.join(sourcemod_home, "**/*.inc"), (err, files) => {
      for (let file of files) {
        let completions = new smCompletions.FileCompletions(URI.file(file).toString());
        smParser.parse_file(file, completions, this.definitionsProvider.definitions, true);

        let uri =
          "file://__sourcemod_builtin/" + path.relative(sourcemod_home, file);
        this.completionsProvider.completions.set(uri, completions);
      }
    });
  }
}
