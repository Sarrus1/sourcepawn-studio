import * as vscode from "vscode";
import * as glob from "glob";
import * as path from "path";
import { URI } from "vscode-uri";
import * as fs from "fs";
import * as smCompletions from "./spCompletions";
import * as spDocCompletions from "./spDocCompletions";
import * as smDefinitions from "./spDefinitions";
import * as smParser from "./spParser";

export class Providers {
  completionsProvider: smCompletions.CompletionRepository;
	documentationProvider: spDocCompletions.JsDocCompletionProvider;
  definitionsProvider: smDefinitions.DefinitionRepository;
	hoverProvider: smCompletions.CompletionRepository;

  constructor(globalState?: vscode.Memento) {
		let CompletionRepo = new smCompletions.CompletionRepository(
      globalState
    );
    this.completionsProvider = CompletionRepo;
    this.definitionsProvider = new smDefinitions.DefinitionRepository(
      globalState
    );
		this.hoverProvider = CompletionRepo;
		this.documentationProvider = new spDocCompletions.JsDocCompletionProvider;
  }

  public handle_added_document(event : vscode.FileCreateEvent) {
    for(let file of event.files)
    {
      this.completionsProvider.documents.set(path.basename(file.fsPath), file);
    }
  }

	public handle_document_change(event: vscode.TextDocumentChangeEvent) {
    let this_completions : smCompletions.FileCompletions = new smCompletions.FileCompletions(event.document.uri.toString());
		let file_path : string = event.document.uri.fsPath;
    this.completionsProvider.documents.set(path.basename(file_path), event.document.uri);
		// Some file paths are appened with .git
		file_path = file_path.replace(".git", "");
    // We use parse_text here, otherwise, if the user didn't save the file, the changes wouldn't be registered.
		try{
			smParser.parse_text(event.document.getText(), file_path, this_completions, this.definitionsProvider.definitions, this.completionsProvider.documents);
		}
		catch(error){console.log(error)}
		this.read_unscanned_imports(this_completions);
    this.completionsProvider.completions.set(event.document.uri.toString(), this_completions);
  }

  public handle_new_document(document: vscode.TextDocument) {
    let this_completions : smCompletions.FileCompletions = new smCompletions.FileCompletions(document.uri.toString());
		let file_path : string =document.uri.fsPath;
		if(path.extname(file_path)=="git") return;
    this.completionsProvider.documents.set(path.basename(file_path), document.uri);
		// Some file paths are appened with .git
		//file_path = file_path.replace(".git", "");
		try{
			smParser.parse_file(file_path, this_completions, this.definitionsProvider.definitions, this.completionsProvider.documents);
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
			smParser.parse_file(path, this_completions, this.definitionsProvider.definitions, this.completionsProvider.documents);
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
          let new_completions : smCompletions.FileCompletions = new smCompletions.FileCompletions(import_file.uri);
          smParser.parse_file(file, new_completions, this.definitionsProvider.definitions, this.completionsProvider.documents, import_file.IsBuiltIn);
					this.read_unscanned_imports(new_completions);
					this.completionsProvider.completions.set(import_file.uri, new_completions);
        }
      }
    }
  }

	public parse_sm_api(SourcemodHome: string): void {
    if (!SourcemodHome) return;
    glob(path.join(SourcemodHome, "**/*.inc"), (err, files) => {
      for (let file of files) {
        let completions = new smCompletions.FileCompletions(URI.file(file).toString());
        smParser.parse_file(file, completions, this.definitionsProvider.definitions, this.completionsProvider.documents, true);

        let uri =
          "file://__sourcemod_builtin/" + path.relative(SourcemodHome, file);
        this.completionsProvider.completions.set(uri, completions);
      }
    });
  }
}
