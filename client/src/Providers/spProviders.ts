import {
  FileCreateEvent,
  TextDocumentChangeEvent,
  Uri,
  window,
  commands,
  workspace as Workspace,
  Memento,
  TextDocument,
  Position,
  CancellationToken,
  CompletionList,
  CompletionItemKind,
  Hover,
  SignatureHelp,
  Location,
  DefinitionLink,
  ProviderResult,
  SemanticTokens,
  SemanticTokensBuilder,
  DocumentSymbol,
  SymbolKind,
} from "vscode";
import * as glob from "glob";
import { basename, extname, join, relative } from "path";
import { URI } from "vscode-uri";
import { existsSync, fstat } from "fs";
import { ItemsRepository, FileItems } from "./spItemsRepository";
import { Include, SPItem } from "./spItems";
import { JsDocCompletionProvider } from "./spDocCompletions";
import { parseText, parseFile } from "./spParser";
import { GetLastFuncName, getLastEnumStructName } from "./spDefinitions";
import { SP_LEGENDS } from "../spLegends";
import { getSignatureAttributes } from "./spSignatures";

export class Providers {
  highlightsProvider: ItemsRepository;
  documentationProvider: JsDocCompletionProvider;
  hoverProvider: ItemsRepository;
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
    let this_completions: FileItems = new FileItems(
      event.document.uri.toString()
    );
    let file_path: string = event.document.uri.fsPath;
    this.itemsRepository.documents.set(
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
        this.itemsRepository
      );
    } catch (error) {
      console.log(error);
    }
    this.readUnscannedImports(this_completions.includes);
    this.itemsRepository.completions.set(
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
    this.itemsRepository.documents.set(basename(file_path), uri.toString());
    try {
      parseFile(file_path, this_completions, this.itemsRepository);
    } catch (error) {
      console.log(error);
    }

    this.readUnscannedImports(this_completions.includes);
    this.itemsRepository.completions.set(uri.toString(), this_completions);
  }

  public handle_document_opening(path: string) {
    let uri: string = URI.file(path).toString();
    let this_completions: FileItems = new FileItems(uri);
    // Some file paths are appened with .git
    path = path.replace(".git", "");
    try {
      parseFile(path, this_completions, this.itemsRepository);
    } catch (error) {
      console.log(error);
    }

    this.readUnscannedImports(this_completions.includes);
    this.itemsRepository.completions.set(uri, this_completions);
  }

  public readUnscannedImports(includes: Include[]) {
    let debugSetting = Workspace.getConfiguration("sourcepawn").get(
      "trace.server"
    );
    let debug = debugSetting == "messages" || debugSetting == "verbose";
    for (let include of includes) {
      if (debug) console.log(include.uri.toString());
      let completion = this.itemsRepository.completions.get(include.uri);
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
              this.itemsRepository,
              include.IsBuiltIn
            );
          } catch (err) {
            console.error(err, include.uri.toString());
          }
          if (debug) console.log("parsed", include.uri.toString());
          this.itemsRepository.completions.set(include.uri, new_completions);
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
        parseFile(file, completions, this.itemsRepository, true);

        let uri = "file://__sourcemod_builtin/" + relative(sm_home, file);
        this.itemsRepository.completions.set(uri, completions);
        this.itemsRepository.documents.set(file, uri);
      } catch (e) {
        console.error(e);
      }
    }
  }

  public provideCompletionItems(
    document: TextDocument,
    position: Position,
    token: CancellationToken
  ): CompletionList {
    const text = document
      .lineAt(position.line)
      .text.substr(0, position.character);
    // If the trigger char is a space, check if there is a new behind, as this block deals with the new declaration.
    if (text[text.length - 1] === " ") {
      if (position.character > 0) {
        const line = document
          .lineAt(position.line)
          .text.substr(0, position.character);
        let match = line.match(/new\s*\w*$/);
        if (match) {
          let items = this.itemsRepository
            .getAllItems(document.uri.toString())
            .filter(
              (item) =>
                item.kind === CompletionItemKind.Method &&
                item.name === item.parent
            );
          return new CompletionList(
            items.map((e) => e.toCompletionItem(document.uri.fsPath))
          );
        }
      }
      return undefined;
    }
    let match = text.match(/^\s*#\s*include\s*(<[^>]*|"[^"]*)$/);
    if (match) {
      return this.itemsRepository.getIncludeCompletions(document, match[1]);
    }
    match = text.match(
      /^\s*(?:HookEvent|HookEventEx)\s*\(\s*(\"[^\"]*|\'[^\']*)$/
    );
    if (match) {
      return this.itemsRepository.getEventCompletions();
    }
    if (['"', "'", "<", "/", "\\"].includes(text[text.length - 1]))
      return undefined;
    if (/[^:]\:$/.test(text)) {
      return undefined;
    }
    return this.itemsRepository.getCompletions(document, position);
  }

  provideHover(
    document: TextDocument,
    position: Position,
    token: CancellationToken
  ): Hover {
    let item = this.itemsRepository.getItemFromPosition(document, position);
    if (typeof item !== "undefined") {
      return item.toHover();
    }
    return undefined;
  }

  provideSignatureHelp(
    document: TextDocument,
    position: Position,
    token: CancellationToken
  ): SignatureHelp {
    let blankReturn = {
      signatures: [],
      activeSignature: 0,
      activeParameter: 0,
    };
    let { croppedLine, parameterCount } = getSignatureAttributes(
      document,
      position
    );
    if (typeof croppedLine === "undefined") {
      return blankReturn;
    }
    // Check if it's a method
    let match = croppedLine.match(/\.(\w+)$/);
    if (match) {
      let methodName = match[1];
      let allItems = this.itemsRepository.getAllItems(document.uri.toString());
      let lastFuncName = GetLastFuncName(position, document);
      let newPos = new Position(1, croppedLine.length);
      let lastEnumStruct = getLastEnumStructName(position, document);
      let type = this.itemsRepository.getTypeOfVariable(
        croppedLine,
        newPos,
        allItems,
        lastFuncName,
        lastEnumStruct
      );
      let variableTypes: string[] = this.itemsRepository.getAllInheritances(
        type,
        allItems
      );
      let items = this.itemsRepository
        .getAllItems(document.uri.toString())
        .filter(
          (item) =>
            (item.kind === CompletionItemKind.Method ||
              item.kind === CompletionItemKind.Property) &&
            variableTypes.includes(item.parent) &&
            item.name === methodName
        );
      return {
        signatures: items.map((e) => e.toSignature()),
        activeParameter: parameterCount,
        activeSignature: 0,
      };
    }
    // Match for new keywords
    match = croppedLine.match(/new\s+(\w+)/);
    if (match) {
      let methodMapName = match[1];
      let items = this.itemsRepository
        .getAllItems(document.uri.toString())
        .filter(
          (item) =>
            item.kind === CompletionItemKind.Method &&
            item.name === methodMapName &&
            item.parent === methodMapName
        );
      return {
        signatures: items.map((e) => e.toSignature()),
        activeParameter: parameterCount,
        activeSignature: 0,
      };
    }

    match = croppedLine.match(/(\w+)$/);
    if (!match) {
      return blankReturn;
    }
    if (["if", "for", "while", "case", "switch", "return"].includes(match[1])) {
      return blankReturn;
    }
    let item = this.itemsRepository
      .getAllItems(document.uri.toString())
      .find(
        (item) =>
          item.name === match[1] && item.kind === CompletionItemKind.Function
      );
    if (typeof item === "undefined") {
      return blankReturn;
    }
    return {
      signatures: [item.toSignature()],
      activeParameter: parameterCount,
      activeSignature: 0,
    };
  }

  public provideDefinition(
    document: TextDocument,
    position: Position,
    token: CancellationToken
  ): Location | DefinitionLink[] {
    let item = this.itemsRepository.getItemFromPosition(document, position);
    if (typeof item !== "undefined") {
      return item.toDefinitionItem();
    }
    return undefined;
  }

  public provideDocumentSemanticTokens(
    document: TextDocument
  ): ProviderResult<SemanticTokens> {
    const tokensBuilder = new SemanticTokensBuilder(SP_LEGENDS);
    let allItems: SPItem[] = this.itemsRepository.getAllItems(
      document.uri.toString()
    );
    for (let item of allItems) {
      if (
        item.kind === CompletionItemKind.Constant ||
        item.kind === CompletionItemKind.EnumMember
      ) {
        for (let call of item.calls) {
          if (call.uri.fsPath === document.uri.fsPath) {
            tokensBuilder.push(call.range, "variable", ["readonly"]);
          }
        }
      }
    }
    return tokensBuilder.build();
  }

  public provideDocumentSymbols(
    document: TextDocument,
    token: CancellationToken
  ): DocumentSymbol[] {
    let symbols: DocumentSymbol[] = [];
    const allowedKinds = [
      CompletionItemKind.Function,
      CompletionItemKind.Class,
      CompletionItemKind.Struct,
    ];
    let items = this.itemsRepository.getAllItems(document.uri.toString());
    let file = document.uri.fsPath;
    for (let item of items) {
      if (allowedKinds.includes(item.kind) && item.file === file) {
        let symbol = item.toDocumentSymbol();
        if (file.includes("convars.inc")) {
          console.debug(symbol);
        }

        if (
          item.kind === CompletionItemKind.Struct &&
          typeof symbol !== "undefined"
        ) {
          let childrens: DocumentSymbol[] = [];
          for (let subItem of items) {
            if (
              subItem.kind === CompletionItemKind.Method &&
              subItem.file === file &&
              subItem.parent === item.name
            ) {
              let children = subItem.toDocumentSymbol();
              if (typeof children !== "undefined") {
                childrens.push(children);
              }
            }
          }
          symbol.children = childrens;
        }
        if (typeof symbol !== "undefined") {
          symbols.push(symbol);
        }
      }
    }
    return symbols;
  }
}
