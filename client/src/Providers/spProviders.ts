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
  SemanticTokens,
  SemanticTokensBuilder,
  DocumentSymbol,
  Definition,
  LocationLink,
} from "vscode";
import * as glob from "glob";
import { extname, join, relative } from "path";
import { URI } from "vscode-uri";
import { existsSync } from "fs";
import { ItemsRepository, FileItems } from "./spItemsRepository";
import { Include, SPItem } from "./spItems";
import { JsDocCompletionProvider } from "./spDocCompletions";
import { parseText, parseFile } from "./spParser";
import {
  GetLastFuncName,
  getLastEnumStructNameOrMethodMap as getLastEnumStructOrMethodMap,
} from "./spDefinitions";
import { SP_LEGENDS } from "../spLegends";
import { getSignatureAttributes } from "./spSignatures";
import { globalIdentifier } from "./spGlobalIdentifier";

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
    this.itemsRepository.completions.set(uri.toString(), this_completions);
  }

  public handle_document_opening(path: string) {
    let uri: string = URI.file(path).toString();
    if (this.itemsRepository.completions.has(uri)) {
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
    let debugSetting = Workspace.getConfiguration("sourcepawn").get(
      "trace.server"
    );
    let debug = debugSetting == "messages" || debugSetting == "verbose";
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
    if (debug) console.log("Parsing SM API");
    let files = glob.sync(join(sm_home, "**/*.inc"));
    for (let file of files) {
      try {
        if (debug) console.log("SM API Reading", file);
        let completions = new FileItems(URI.file(file).toString());
        parseFile(file, completions, this.itemsRepository, true);
        if (debug) console.log("SM API Done parsing", file);

        let uri =
          "file://__sourcemod_builtin/" +
          relative(sm_home, file).replace("\\", "/");
        this.itemsRepository.completions.set(uri, completions);
        this.itemsRepository.documents.add(uri);
        if (debug) console.log("SM API Done dealing with", uri);
      } catch (e) {
        console.error(e);
      }
    }
    if (debug) console.log("Done parsing SM API");
  }

  public async provideCompletionItems(
    document: TextDocument,
    position: Position,
    token: CancellationToken
  ): Promise<CompletionList> {
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
            .filter((item) => item.kind === CompletionItemKind.Constructor);
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

  public async provideHover(
    document: TextDocument,
    position: Position,
    token: CancellationToken
  ): Promise<Hover> {
    let items = this.itemsRepository.getItemFromPosition(document, position);
    if (items.length > 0) {
      return items[0].toHover();
    }
    return undefined;
  }

  public async provideSignatureHelp(
    document: TextDocument,
    position: Position,
    token: CancellationToken
  ): Promise<SignatureHelp> {
    let blankReturn = {
      signatures: [],
      activeSignature: 0,
      activeParameter: 0,
    };
    let { croppedLine, parameterCount } = getSignatureAttributes(
      document,
      position
    );
    if (croppedLine === undefined) {
      return blankReturn;
    }
    // Check if it's a method
    let match = croppedLine.match(/\.(\w+)$/);
    if (match) {
      let methodName = match[1];
      let allItems = this.itemsRepository.getAllItems(document.uri.toString());
      let lastFuncName = GetLastFuncName(position, document, allItems);
      let newPos = new Position(1, croppedLine.length);
      let lastEnumStructOrMethodMap = getLastEnumStructOrMethodMap(
        position,
        document,
        allItems
      );
      let type = this.itemsRepository.getTypeOfVariable(
        croppedLine,
        newPos,
        allItems,
        lastFuncName,
        lastEnumStructOrMethodMap
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
    let items = this.itemsRepository
      .getAllItems(document.uri.toString())
      .filter(
        (item) =>
          item.name === match[1] &&
          [CompletionItemKind.Function, CompletionItemKind.Interface].includes(
            item.kind
          )
      );
    if (items === undefined) {
      return blankReturn;
    }
    // Sort by size of description
    items = items.sort((a, b) => b.description.length - a.description.length);
    return {
      signatures: items.map((e) => e.toSignature()),
      activeParameter: parameterCount,
      activeSignature: 0,
    };
  }

  public async provideDefinition(
    document: TextDocument,
    position: Position,
    token: CancellationToken
  ): Promise<Definition | LocationLink[]> {
    let items = this.itemsRepository.getItemFromPosition(document, position);
    return items.map((e) => e.toDefinitionItem());
  }

  public async provideDocumentSemanticTokens(
    document: TextDocument
  ): Promise<SemanticTokens> {
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

  public async provideDocumentSymbols(
    document: TextDocument,
    token: CancellationToken
  ): Promise<DocumentSymbol[]> {
    let symbols: DocumentSymbol[] = [];
    const allowedKinds = [
      CompletionItemKind.Function,
      CompletionItemKind.Class,
      CompletionItemKind.Struct,
      CompletionItemKind.Enum,
      CompletionItemKind.Constant,
      CompletionItemKind.Variable,
      CompletionItemKind.TypeParameter,
    ];
    const allowedParentsKinds = [
      CompletionItemKind.Class,
      CompletionItemKind.Struct,
      CompletionItemKind.Function,
      CompletionItemKind.Enum,
    ];
    const allowedChildrendKinds = [
      CompletionItemKind.Method,
      CompletionItemKind.Property,
      CompletionItemKind.Variable,
      CompletionItemKind.EnumMember,
    ];
    let items = this.itemsRepository.getAllItems(document.uri.toString());
    let file = document.uri.fsPath;
    for (let item of items) {
      if (allowedKinds.includes(item.kind) && item.file === file) {
        // Don't add non global variables here
        if (
          item.kind === CompletionItemKind.Variable &&
          item.parent !== globalIdentifier
        ) {
          continue;
        }
        let symbol = item.toDocumentSymbol();

        // Check if the item can have childrens
        if (allowedParentsKinds.includes(item.kind) && symbol !== undefined) {
          let childrens: DocumentSymbol[] = [];
          // Iterate over all items to get the childrens
          for (let subItem of items) {
            if (
              allowedChildrendKinds.includes(subItem.kind) &&
              subItem.file === file &&
              subItem.parent === item.name
            ) {
              let children = subItem.toDocumentSymbol();
              if (children !== undefined) {
                childrens.push(children);
              }
            }
          }
          symbol.children = childrens;
        }
        if (symbol !== undefined) {
          symbols.push(symbol);
        }
      }
    }
    return symbols;
  }
}
