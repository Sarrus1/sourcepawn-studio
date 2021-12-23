import {
  FileCreateEvent,
  TextDocumentChangeEvent,
  Uri,
  workspace as Workspace,
  Memento,
  TextDocument,
  Position,
  CancellationToken,
  CompletionList,
  Hover,
  SignatureHelp,
  SemanticTokens,
  DocumentSymbol,
  Definition,
  LocationLink,
} from "vscode";
import { extname } from "path";
import { URI } from "vscode-uri";
import { existsSync } from "fs";
import { ItemsRepository } from "./spItemsRepository";
import { FileItems } from "./spFilesRepository";
import { Include } from "./spItems";
import { JsDocCompletionProvider } from "../Providers/spDocCompletions";
import { parseText, parseFile } from "../Parser/spParser";
import { definitionsProvider } from "../Providers/spDefinitionProvider";
import { signatureProvider } from "../Providers/spSignatureProvider";
import { hoverProvider } from "../Providers/spHoverProvider";
import { symbolProvider } from "../Providers/spSymbolProvider";
import { completionProvider } from "../Providers/spCompletionProvider";
import { semanticTokenProvider } from "../Providers/spSemanticTokenProvider";

export class Providers {
  documentationProvider: JsDocCompletionProvider;
  itemsRepository: ItemsRepository;

  constructor(globalState?: Memento) {
    this.documentationProvider = new JsDocCompletionProvider();
    this.itemsRepository = new ItemsRepository(globalState);
  }

  public handleAddedDocument(event: FileCreateEvent) {
    for (let file of event.files) {
      this.newDocumentCallback(URI.file(file.fsPath));
    }
  }

  public handleDocumentChange(event: TextDocumentChangeEvent) {
    if (event.contentChanges.length > 0) {
      let textChange = event.contentChanges[0].text;
      // Don't parse the document every character changes.
      if (/\w+/.test(textChange)) {
        return;
      }
    }

    let this_completions: FileItems = new FileItems(
      event.document.uri.toString()
    );
    let file_path: string = event.document.uri.fsPath;
    this.itemsRepository.documents.add(event.document.uri.toString());
    // Some file paths are appened with .git
    file_path = file_path.replace(".git", "");
    // We use parse_text here, otherwise, if the user didn't save the file, the changes wouldn't be registered.
    try {
      parseText(
        event.document.getText(),
        file_path,
        this_completions,
        this.itemsRepository
      );
    } catch (error) {
      console.log(error);
    }
    this.readUnscannedImports(this_completions.includes);
    this.itemsRepository.items.set(
      event.document.uri.toString(),
      this_completions
    );
  }

  public handleNewDocument(document: TextDocument) {
    this.newDocumentCallback(document.uri);
  }

  public newDocumentCallback(uri: Uri) {
    let ext: string = extname(uri.fsPath);
    if (ext != ".inc" && ext != ".sp") {
      return;
    }
    let this_completions: FileItems = new FileItems(uri.toString());
    let file_path: string = uri.fsPath;
    // Some file paths are appened with .git
    if (file_path.includes(".git")) {
      return;
    }
    this.itemsRepository.documents.add(uri.toString());
    try {
      parseFile(file_path, this_completions, this.itemsRepository);
    } catch (error) {
      console.log(error);
    }

    this.readUnscannedImports(this_completions.includes);
    this.itemsRepository.items.set(uri.toString(), this_completions);
  }

  public handle_document_opening(path: string) {
    let uri: string = URI.file(path).toString();
    if (this.itemsRepository.items.has(uri)) {
      return;
    }
    let this_completions: FileItems = new FileItems(uri);
    // Some file paths are appened with .git
    path = path.replace(".git", "");
    try {
      parseFile(path, this_completions, this.itemsRepository);
    } catch (error) {
      console.log(error);
    }

    this.readUnscannedImports(this_completions.includes);
    this.itemsRepository.items.set(uri, this_completions);
  }

  public readUnscannedImports(includes: Include[]) {
    let debugSetting = Workspace.getConfiguration("sourcepawn").get(
      "trace.server"
    );
    let debug = debugSetting == "messages" || debugSetting == "verbose";
    for (let include of includes) {
      if (debug) console.log(include.uri.toString());
      let completion = this.itemsRepository.items.get(include.uri);
      if (completion === undefined) {
        if (debug) console.log("reading", include.uri.toString());
        let file = URI.parse(include.uri).fsPath;
        if (existsSync(file)) {
          if (debug) console.log("found", include.uri.toString());
          let new_completions: FileItems = new FileItems(include.uri);
          try {
            parseFile(
              file,
              new_completions,
              this.itemsRepository,
              include.IsBuiltIn
            );
          } catch (err) {
            console.error(err, include.uri.toString());
          }
          if (debug) console.log("parsed", include.uri.toString());
          this.itemsRepository.items.set(include.uri, new_completions);
          if (debug) console.log("added", include.uri.toString());
          this.readUnscannedImports(new_completions.includes);
        }
      }
    }
  }

  public async provideCompletionItems(
    document: TextDocument,
    position: Position,
    token: CancellationToken
  ): Promise<CompletionList> {
    return completionProvider(this.itemsRepository, document, position, token);
  }

  public async provideHover(
    document: TextDocument,
    position: Position,
    token: CancellationToken
  ): Promise<Hover> {
    return hoverProvider(this.itemsRepository, document, position, token);
  }

  public async provideSignatureHelp(
    document: TextDocument,
    position: Position,
    token: CancellationToken
  ): Promise<SignatureHelp> {
    return signatureProvider(this.itemsRepository, document, position, token);
  }

  public async provideDefinition(
    document: TextDocument,
    position: Position,
    token: CancellationToken
  ): Promise<Definition | LocationLink[]> {
    return definitionsProvider(this.itemsRepository, document, position, token);
  }

  public async provideDocumentSemanticTokens(
    document: TextDocument
  ): Promise<SemanticTokens> {
    return semanticTokenProvider(this.itemsRepository, document);
  }

  public async provideDocumentSymbols(
    document: TextDocument,
    token: CancellationToken
  ): Promise<DocumentSymbol[]> {
    return symbolProvider(this.itemsRepository, document, token);
  }
}
