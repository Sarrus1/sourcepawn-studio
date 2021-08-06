import {
  workspace as Workspace,
  FileCreateEvent,
  TextDocumentChangeEvent,
  Uri,
  Memento,
  window,
  commands,
  TextDocument,
} from "vscode";
import * as glob from "glob";
import { basename, extname, join, relative } from "path";
import { URI } from "vscode-uri";
import { existsSync } from "fs";
import { ItemsRepository, FileItems } from "./spItemsRepository";
import { Include } from "./spItems";
import { JsDocCompletionProvider } from "./spDocCompletions";
import { parseText, parseFile } from "./spParser";

export class Providers {
  completionsProvider: ItemsRepository;
  documentationProvider: JsDocCompletionProvider;
  hoverProvider: ItemsRepository;

  constructor(globalState?: Memento) {
    let CompletionRepo = new ItemsRepository(globalState);
    this.completionsProvider = CompletionRepo;
    this.hoverProvider = CompletionRepo;
    this.documentationProvider = new JsDocCompletionProvider();
  }

  public handleAddedDocument(event: FileCreateEvent) {
    for (let file of event.files) {
      this.newDocumentCallback(URI.file(file.fsPath));
    }
  }

  public handleDocumentChange(event: TextDocumentChangeEvent) {
    let this_completions: FileItems = new FileItems(
      event.document.uri.toString()
    );
    let file_path: string = event.document.uri.fsPath;
    this.completionsProvider.documents.set(
      basename(file_path),
      event.document.uri.toString()
    );
    // Some file paths are appened with .git
    file_path = file_path.replace(".git", "");
    // We use parse_text here, otherwise, if the user didn't save the file, the changes wouldn't be registered.
    try {
      parseText(
        event.document.getText(),
        file_path,
        this_completions,
        this.completionsProvider.documents
      );
    } catch (error) {
      console.log(error);
    }
    this.readUnscannedImports(this_completions.includes);
    this.completionsProvider.completions.set(
      event.document.uri.toString(),
      this_completions
    );
  }

  public handleNewDocument(document: TextDocument) {
    this.newDocumentCallback(document.uri);
  }

  public newDocumentCallback(uri: Uri) {
    let ext: string = extname(uri.fsPath);
    if (ext != ".inc" && ext != ".sp") return;
    let this_completions: FileItems = new FileItems(uri.toString());
    let file_path: string = uri.fsPath;
    // Some file paths are appened with .git
    if (file_path.includes(".git")) return;
    this.completionsProvider.documents.set(basename(file_path), uri.toString());
    try {
      parseFile(
        file_path,
        this_completions,
        this.completionsProvider.documents
      );
    } catch (error) {
      console.log(error);
    }

    this.readUnscannedImports(this_completions.includes);
    this.completionsProvider.completions.set(uri.toString(), this_completions);
  }

  public handle_document_opening(path: string) {
    let uri: string = URI.file(path).toString();
    let this_completions: FileItems = new FileItems(uri);
    // Some file paths are appened with .git
    path = path.replace(".git", "");
    try {
      parseFile(path, this_completions, this.completionsProvider.documents);
    } catch (error) {
      console.log(error);
    }

    this.readUnscannedImports(this_completions.includes);
    this.completionsProvider.completions.set(uri, this_completions);
  }

  public readUnscannedImports(includes: Include[]) {
    let debugSetting = Workspace.getConfiguration("sourcepawn").get(
      "trace.server"
    );
    let debug = debugSetting == "messages" || debugSetting == "verbose";
    for (let include of includes) {
      if (debug) console.log(include.uri.toString());
      let completion = this.completionsProvider.completions.get(include.uri);
      if (typeof completion === "undefined") {
        if (debug) console.log("reading", include.uri.toString());
        let file = URI.parse(include.uri).fsPath;
        if (existsSync(file)) {
          if (debug) console.log("found", include.uri.toString());
          let new_completions: FileItems = new FileItems(include.uri);
          try {
            parseFile(
              file,
              new_completions,
              this.completionsProvider.documents,
              include.IsBuiltIn
            );
          } catch (err) {
            console.error(err, include.uri.toString());
          }
          if (debug) console.log("parsed", include.uri.toString());
          this.completionsProvider.completions.set(
            include.uri,
            new_completions
          );
          if (debug) console.log("added", include.uri.toString());
          this.readUnscannedImports(new_completions.includes);
        }
      }
    }
  }

  public parseSMApi(): void {
    let sm_home: string =
      Workspace.getConfiguration("sourcepawn").get("SourcemodHome") || "";
    if (sm_home == "") {
      window
        .showWarningMessage(
          "SourceMod API not found in the project. You should set SourceMod Home for tasks generation to work. Do you want to install it automatically?",
          "Yes",
          "No, open Settings"
        )
        .then((choice) => {
          if (choice == "Yes") {
            commands.executeCommand("sourcepawn-installSM");
          } else if (choice === "No, open Settings") {
            commands.executeCommand(
              "workbench.action.openSettings",
              "@ext:sarrus.sourcepawn-vscode"
            );
          }
        });
      return;
    }
    let files = glob.sync(join(sm_home, "**/*.inc"));
    for (let file of files) {
      try {
        let completions = new FileItems(URI.file(file).toString());
        parseFile(file, completions, this.completionsProvider.documents, true);

        let uri = "file://__sourcemod_builtin/" + relative(sm_home, file);
        this.completionsProvider.completions.set(uri, completions);
        this.completionsProvider.documents.set(file, uri);
      } catch (e) {
        console.error(e);
      }
    }
  }
}
