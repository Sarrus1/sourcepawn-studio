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
  Range,
} from "vscode";
import { basename, join } from "path";
import { existsSync } from "fs";
import { URI } from "vscode-uri";
import { SPItem, Include } from "./spItems";
import { events } from "../Misc/sourceEvents";
import {
  GetLastFuncName,
  isFunction,
  getLastEnumStructName,
} from "./spDefinitions";
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
    // If the trigger char is a space, check if there is a new behind, as this block deals with the new declaration.
    if (text[text.length - 1] === " ") {
      if (position.character > 0) {
        const line = document
          .lineAt(position.line)
          .text.substr(0, position.character);
        let match = line.match(/new\s*\w*$/);
        if (match) {
          let items = this.getAllItems(document.uri.toString()).filter(
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
    let line = document.lineAt(position.line).text;
    let is_method = checkIfMethod(line, position);
    let all_completions: SPItem[] = this.getAllItems(document.uri.toString());
    let all_completions_list: CompletionList = new CompletionList();
    if (all_completions !== []) {
      let lastFunc: string = GetLastFuncName(position, document);
      let lastEnumStruct = getLastEnumStructName(position, document);
      if (is_method) {
        let variableType = this.getTypeOfVariable(
          line,
          position,
          all_completions,
          lastFunc,
          lastEnumStruct
        );
        let variableTypes: string[] = this.getAllInheritances(
          variableType,
          all_completions
        );
        for (let item of all_completions) {
          if (
            (item.kind === CompletionItemKind.Method ||
              item.kind === CompletionItemKind.Property) &&
            variableTypes.includes(item.parent) &&
            // Don't include the constructor of the methodmap
            !variableTypes.includes(item.name)
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

  getAllInheritances(variableType: string, allCompletions: SPItem[]): string[] {
    let methodMapItem = allCompletions.find(
      (e) => e.kind === CompletionItemKind.Class && e.name === variableType
    );
    if (
      typeof methodMapItem === "undefined" ||
      typeof methodMapItem.parent === "undefined"
    ) {
      return [variableType];
    }
    return [variableType].concat(
      this.getAllInheritances(methodMapItem.parent, allCompletions)
    );
  }

  getTypeOfVariable(
    line: string,
    position: Position,
    allItems: SPItem[],
    lastFuncName: string,
    lastEnumStruct: string
  ): string {
    let i = position.character - 1;
    let bCounter = 0;
    let pCounter = 0;
    while (i >= 0) {
      if (/\w/.test(line[i])) {
        i--;
      } else if (line[i] === ".") {
        i--;
        break;
      }
    }
    let wordCounter = 0;
    let words: string[] = [""];
    while (i >= 0) {
      if (line[i] === "]") {
        bCounter++;
        i--;
        continue;
      }
      if (line[i] === "[") {
        bCounter--;
        i--;
        continue;
      }
      if (line[i] === ")") {
        pCounter++;
        i--;
        continue;
      }
      if (line[i] === "(") {
        pCounter--;
        i--;
        continue;
      }
      if (bCounter === 0 && pCounter === 0) {
        if (/\w/.test(line[i])) {
          words[wordCounter] = line[i] + words[wordCounter];
        } else if (line[i] === ".") {
          wordCounter++;
          words[wordCounter] = "";
        } else {
          break;
        }
      }
      i--;
    }
    let variableType: string;
    if (lastEnumStruct !== "$GLOBAL" && words[words.length - 1] === "this") {
      variableType = lastEnumStruct;
    } else {
      variableType = allItems.find(
        (e) =>
          (e.kind === CompletionItemKind.Variable &&
            ["$GLOBAL", lastFuncName].includes(e.scope) &&
            e.name === words[words.length - 1]) ||
          (e.kind === CompletionItemKind.Function &&
            e.name === words[words.length - 1])
      ).type;
    }
    if (words.length > 1) {
      words = words.slice(0, words.length - 1).reverse();
      for (let word of words) {
        variableType = allItems.find(
          (e) =>
            (e.kind === CompletionItemKind.Method ||
              e.kind === CompletionItemKind.Property) &&
            e.parent === variableType &&
            e.name === word
        ).type;
      }
    }

    return variableType;
  }

  getAllItems(file: string): SPItem[] {
    let includes = new Set<string>();
    let MainPath: string =
      Workspace.getConfiguration("sourcepawn").get("MainPath") || "";
    let allItems;
    if (MainPath !== "") {
      if (!existsSync(MainPath)) {
        let workspace: WorkspaceFolder = Workspace.workspaceFolders[0];
        MainPath = join(workspace.uri.fsPath, MainPath);
        if (!existsSync(MainPath)) {
          throw "MainPath is incorrect.";
        }
      }
      let uri = URI.file(MainPath).toString();
      allItems = this.completions.get(uri);
      if (!includes.has(uri)) {
        includes.add(uri);
      }
    } else {
      allItems = this.completions.get(file);
      includes.add(file);
    }
    if (typeof allItems !== "undefined") {
      this.getIncludedFiles(allItems, includes);
    } else {
      return [];
    }
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
    let item = this.getItemFromPosition(document, position);
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
      let allItems = this.getAllItems(document.uri.toString());
      let lastFuncName = GetLastFuncName(position, document);
      let newPos = new Position(1, croppedLine.length);
      let lastEnumStruct = getLastEnumStructName(position, document);
      let type = this.getTypeOfVariable(
        croppedLine,
        newPos,
        allItems,
        lastFuncName,
        lastEnumStruct
      );
      let variableTypes: string[] = this.getAllInheritances(type, allItems);
      let items = this.getAllItems(document.uri.toString()).filter(
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
      let items = this.getAllItems(document.uri.toString()).filter(
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
    let item = this.getAllItems(document.uri.toString()).find(
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
    let item = this.getItemFromPosition(document, position);
    if (typeof item !== "undefined") {
      return item.toDefinitionItem();
    }
    return undefined;
  }

  public provideDocumentSemanticTokens(
    document: TextDocument
  ): ProviderResult<SemanticTokens> {
    const tokensBuilder = new SemanticTokensBuilder(SP_LEGENDS);
    let allItems: SPItem[] = this.getAllItems(document.uri.toString());
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

  getItemFromPosition(document: TextDocument, position: Position): SPItem {
    let range = document.getWordRangeAtPosition(position);
    // First check if we are dealing with a method or property.
    let isMethod: boolean = false;
    let isConstructor: boolean = false;
    let match: RegExpMatchArray;
    if (range.start.character > 0) {
      let newPosStart = new Position(
        range.start.line,
        range.start.character - 1
      );
      let newPosEnd = new Position(range.start.line, range.start.character);
      let newRange = new Range(newPosStart, newPosEnd);
      let char = document.getText(newRange);
      isMethod = char === ".";
      if (!isMethod) {
        let newPosStart = new Position(range.start.line, 0);
        let newPosEnd = new Position(range.start.line, range.end.character);
        let newRange = new Range(newPosStart, newPosEnd);
        let line = document.getText(newRange);
        match = line.match(/new\s+(\w+)$/);
        if (match) {
          isConstructor = true;
        }
      }
    }
    let word: string = document.getText(range);
    let allItems = this.getAllItems(document.uri.toString());
    let lastFunc: string = GetLastFuncName(position, document);
    let lastEnumStruct: string = getLastEnumStructName(position, document);

    if (isMethod) {
      let line = document.lineAt(position.line).text;
      // If we are dealing with a method or property, look for the type of the variable
      let variableType = this.getTypeOfVariable(
        line,
        position,
        allItems,
        lastFunc,
        lastEnumStruct
      );
      // Get inheritances from methodmaps
      let variableTypes: string[] = this.getAllInheritances(
        variableType,
        allItems
      );
      // Find and return the matching item
      let item = allItems.find(
        (item) =>
          (item.kind === CompletionItemKind.Method ||
            item.kind === CompletionItemKind.Property) &&
          variableTypes.includes(item.parent) &&
          item.name === word
      );
      return item;
    }

    if (isConstructor) {
      let item = this.getAllItems(document.uri.toString()).find(
        (item) =>
          item.kind === CompletionItemKind.Method &&
          item.name === match[1] &&
          item.parent === match[1]
      );
      return item;
    }
    // Check if we are dealing with a function
    let bIsFunction = isFunction(
      range,
      document,
      document.lineAt(position.line).text.length
    );
    let item = undefined;
    if (bIsFunction) {
      item = allItems.find(
        (item) =>
          item.kind === CompletionItemKind.Function && item.name === word
      );
      return item;
    }
    item = allItems.find(
      (item) =>
        !(
          item.kind === CompletionItemKind.Method ||
          item.kind === CompletionItemKind.Property ||
          item.kind === CompletionItemKind.Function
        ) &&
        item.name === word &&
        item.scope === lastFunc
    );
    if (typeof item !== "undefined") {
      return item;
    }
    item = allItems.find((item) => {
      if (
        item.kind === CompletionItemKind.Method ||
        item.kind === CompletionItemKind.Property
      ) {
        return false;
      }
      if (typeof item.scope !== "undefined") {
        return item.scope === "$GLOBAL" && item.name === word;
      }
      return item.name === word;
    });
    return item;
  }
}

function checkIfMethod(line: string, position: Position): boolean {
  let i = position.character - 1;
  while (i > 0) {
    if (/\w/.test(line[i])) {
      i--;
    } else if (line[i] === ".") {
      return true;
    } else {
      return false;
    }
  }
  return false;
}
