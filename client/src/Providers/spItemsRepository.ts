import {
  CompletionItem,
  workspace as Workspace,
  CompletionItemProvider,
  Memento,
  Disposable,
  TextDocument,
  Position,
  CancellationToken,
  CompletionList,
  CompletionItemKind,
  WorkspaceFolder,
  Hover,
  SignatureHelp,
  Location,
  DefinitionLink,
  ProviderResult,
  SemanticTokens,
  SemanticTokensBuilder,
} from "vscode";
import { basename, join } from "path";
import { existsSync } from "fs";
import { URI } from "vscode-uri";
import { SPItem, Include } from "./spItems";
import { events } from "../Misc/sourceEvents";
import { GetLastFuncName, isFunction } from "./spDefinitions";
import { getSignatureAttributes } from "./spSignatures";
import { SP_LEGENDS } from "../spLegends";

export class FileItems {
  completions: Map<string, SPItem>;
  includes: Include[];
  uri: string;

  constructor(uri: string) {
    this.completions = new Map();
    this.includes = [];
    this.uri = uri;
  }

  add(id: string, completion: SPItem) {
    this.completions.set(id, completion);
  }

  get(id: string): SPItem {
    return this.completions.get(id);
  }

  getCompletions(repo: ItemsRepository): SPItem[] {
    let completions = [];
    for (let completion of this.completions.values()) {
      completions.push(completion);
    }
    return completions;
  }

  toCompletionResolve(item: CompletionItem): CompletionItem {
    item.label = item.label;
    item.documentation = item.documentation;
    return item;
  }

  addInclude(include: string, IsBuiltIn: boolean) {
    this.includes.push(new Include(include, IsBuiltIn));
  }

  resolve_import(
    file: string,
    documents: Map<string, string>,
    IsBuiltIn: boolean = false
  ) {
    let inc_file: string;
    // If no extension is provided, it's a .inc file
    if (!/.sp\s*$/g.test(file) && !/.inc\s*$/g.test(file)) {
      file += ".inc";
    }

    let match = file.match(/include\/(.*)/);
    if (match) file = match[1];
    let uri: string;
    if (!(uri = documents.get(basename(file)))) {
      let includes_dirs: string[] = Workspace.getConfiguration(
        "sourcepawn"
      ).get("optionalIncludeDirsPaths");
      for (let includes_dir of includes_dirs) {
        inc_file = join(includes_dir, file);
        if (existsSync(inc_file)) {
          this.addInclude(URI.file(inc_file).toString(), IsBuiltIn);
          return;
        }
      }
      this.addInclude("file://__sourcemod_builtin/" + file, IsBuiltIn);
    } else {
      this.addInclude(uri, IsBuiltIn);
    }
  }
}

export class ItemsRepository implements CompletionItemProvider, Disposable {
  public completions: Map<string, FileItems>;
  public documents: Map<string, string>;
  private globalState: Memento;

  constructor(globalState?: Memento) {
    this.completions = new Map();
    this.documents = new Map();
    this.globalState = globalState;
  }

  public provideCompletionItems(
    document: TextDocument,
    position: Position,
    token: CancellationToken
  ): CompletionList {
    const text = document
      .lineAt(position.line)
      .text.substr(0, position.character);
    let match = text.match(/^\s*#\s*include\s*(<[^>]*|"[^"]*)$/);
    if (match) {
      return this.getIncludeCompletions(document, match[1]);
    }
    match = text.match(
      /^\s*(?:HookEvent|HookEventEx)\s*\(\s*(\"[^\"]*|\'[^\']*)$/
    );
    if (match) {
      return this.getEventCompletions();
    }
    if (['"', "'", "<", "/", "\\"].includes(text[text.length - 1]))
      return undefined;
    return this.getCompletions(document, position);
  }

  public dispose() {}

  getEventCompletions(): CompletionList {
    return new CompletionList(events);
  }

  getIncludeCompletions(
    document: TextDocument,
    tempName: string
  ): CompletionList {
    let isQuoteInclude: boolean = tempName.includes('"');
    tempName = tempName.replace("<", "").replace('"', "");
    let match = tempName.match(/([^\/]+\/)+/);
    tempName = match ? match[0] : "";
    let scriptingDirname: string = document.uri.toString();
    let itemsNames: string[] = [];
    scriptingDirname =
      scriptingDirname.replace(basename(document.uri.fsPath), "") + "include/";
    let scriptingDirnames: string[] = [scriptingDirname];
    let includes_dirs: string[] = Workspace.getConfiguration("sourcepawn").get(
      "optionalIncludeDirsPaths"
    );
    scriptingDirnames = scriptingDirnames.concat(includes_dirs);
    let items: CompletionItem[] = [];
    let cleanedUri: string;
    for (let uri of this.documents.values()) {
      if (uri.includes("file://__sourcemod_builtin/" + tempName)) {
        cleanedUri = uri.replace("file://__sourcemod_builtin/" + tempName, "");
        let match = cleanedUri.match(/([^\/]+\/)?/);
        if (match[0] != "") {
          let item = {
            label: match[0].replace("/", ""),
            kind: CompletionItemKind.Folder,
            detail: "Sourcemod BuiltIn",
          };
          if (itemsNames.indexOf(match[0]) == -1) {
            items.push(item);
            itemsNames.push(match[0]);
          }
        } else {
          let insertText = cleanedUri.replace(".inc", "");
          insertText += isQuoteInclude ? "" : ">";
          let item = {
            label: cleanedUri,
            kind: CompletionItemKind.File,
            detail: "Sourcemod BuiltIn",
            insertText: insertText,
          };
          if (itemsNames.indexOf(cleanedUri) == -1) {
            items.push(item);
            itemsNames.push(cleanedUri);
          }
        }
      } else {
        for (scriptingDirname of scriptingDirnames) {
          if (uri.includes(scriptingDirname + tempName)) {
            cleanedUri = uri.replace(scriptingDirname + tempName, tempName);
            let match = cleanedUri.match(/([^\/]+\/)?/);
            if (match[0] != "") {
              let item = {
                label: match[0].replace("/", ""),
                kind: CompletionItemKind.Folder,
                detail: URI.parse(uri).fsPath,
              };
              if (itemsNames.indexOf(match[0]) == -1) {
                items.push(item);
                itemsNames.push(match[0]);
              }
            } else {
              let insertText = cleanedUri.replace(".inc", "");
              insertText += isQuoteInclude ? "" : ">";
              let item = {
                label: cleanedUri,
                kind: CompletionItemKind.File,
                detail: URI.parse(uri).fsPath,
                insertText: insertText,
              };
              if (itemsNames.indexOf(cleanedUri) == -1) {
                items.push(item);
                itemsNames.push(cleanedUri);
              }
            }
          }
        }
      }
    }

    return new CompletionList(items);
  }

  getCompletions(document: TextDocument, position: Position): CompletionList {
    let line = document.lineAt(position.line).text.trim();
    let is_method = line[position.character - 1] === ".";
    let all_completions: SPItem[] = this.getAllItems(document.uri.toString());
    let all_completions_list: CompletionList = new CompletionList();
    if (all_completions != []) {
      let lastFunc: string = GetLastFuncName(position.line, document);
      if (is_method) {
        for (let item of all_completions) {
          if (
            item.kind === CompletionItemKind.Method ||
            item.kind === CompletionItemKind.Property
          ) {
            all_completions_list.items.push(
              item.toCompletionItem(document.uri.fsPath, lastFunc)
            );
          }
        }
        return all_completions_list;
      }
      for (let item of all_completions) {
        if (
          !(
            item.kind === CompletionItemKind.Method ||
            item.kind === CompletionItemKind.Property
          )
        ) {
          all_completions_list.items.push(
            item.toCompletionItem(document.uri.fsPath, lastFunc)
          );
        }
      }
      return all_completions_list;
    }
  }

  getAllItems(file: string): SPItem[] {
    let includes = new Set<string>();
    let MainPath: string =
      Workspace.getConfiguration("sourcepawn").get("MainPath") || "";
    if (MainPath != "") {
      if (!existsSync(MainPath)) {
        let workspace: WorkspaceFolder = Workspace.workspaceFolders[0];
        MainPath = join(workspace.uri.fsPath, MainPath);
        if (!existsSync(MainPath)) {
          throw "MainPath is incorrect.";
        }
      }
      let MainCompletion = this.completions.get(URI.file(MainPath).toString());
      if (MainCompletion) {
        this.getIncludedFiles(MainCompletion, includes);
      }
      let uri = URI.file(MainPath).toString();
      if (!includes.has(uri)) {
        includes.add(uri);
      }
    }
    let completion = this.completions.get(file);

    if (typeof completion !== "undefined") {
      this.getIncludedFiles(completion, includes);
    } else {
      return [];
    }
    includes.add(file);
    return [...includes]
      .map((file) => {
        return this.getFileItems(file);
      })
      .reduce(
        (completion, file_completions) => completion.concat(file_completions),
        []
      );
  }

  getFileItems(file: string): SPItem[] {
    let file_completions: FileItems = this.completions.get(file);
    let completion_list: SPItem[] = [];
    if (file_completions) {
      return file_completions.getCompletions(this);
    }
    return completion_list;
  }

  getIncludedFiles(completions: FileItems, files: Set<string>) {
    for (let include of completions.includes) {
      if (!files.has(include.uri)) {
        files.add(include.uri);
        let include_completions = this.completions.get(include.uri);
        if (include_completions) {
          this.getIncludedFiles(include_completions, files);
        }
      }
    }
  }

  provideHover(
    document: TextDocument,
    position: Position,
    token: CancellationToken
  ): Hover {
    let range = document.getWordRangeAtPosition(position);
    let word = document.getText(range);
    let item = this.getAllItems(document.uri.toString()).find(
      (item) => item.name === word
    );
    return item.toHover();
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
    let { functionName, parameterCount } = getSignatureAttributes(
      document,
      position
    );
    if (typeof functionName === "undefined") {
      return blankReturn;
    }
    let completions = this.getAllItems(document.uri.toString()).find(
      (completion) => completion.name === functionName
    );
    return {
      signatures: [completions.toSignature()],
      activeParameter: parameterCount,
      activeSignature: 0,
    };
  }

  public provideDefinition(
    document: TextDocument,
    position: Position,
    token: CancellationToken
  ): Location | DefinitionLink[] {
    let range = document.getWordRangeAtPosition(position);
    let word: string = document.getText(range);
    let definitions = this.getAllItems(document.uri.toString()).filter(
      (completion) => {
        return completion.name === word;
      }
    );
    let bIsFunction = isFunction(
      range,
      document,
      document.lineAt(position.line).text.length
    );
    let definition = undefined;
    if (bIsFunction) {
      definition = definitions.find(
        (def) => def.kind === CompletionItemKind.Function
      );
      if (typeof definition !== "undefined") {
        return definition.toDefinitionItem();
      }
    }
    let lastFuncName: string = GetLastFuncName(position.line, document);
    definition = definitions.find((def) => def.scope === lastFuncName);
    if (typeof definition !== "undefined") {
      return definition.toDefinitionItem();
    }
    return definitions[0].toDefinitionItem();
  }

  public provideDocumentSemanticTokens(
    document: TextDocument
  ): ProviderResult<SemanticTokens> {
    const tokensBuilder = new SemanticTokensBuilder(SP_LEGENDS);
    let allItems: SPItem[] = this.getAllItems(document.uri.toString());
    for (let item of allItems) {
      if (item.kind === CompletionItemKind.Constant) {
        for (let call of item.calls) {
          if (call.uri.fsPath === document.uri.fsPath) {
            tokensBuilder.push(call.range, "variable", ["readonly"]);
          }
        }
      }
    }
    return tokensBuilder.build();
  }
}
