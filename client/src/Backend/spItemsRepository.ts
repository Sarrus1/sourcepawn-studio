import {
  workspace as Workspace,
  Memento,
  Disposable,
  TextDocument,
  Position,
  CompletionList,
  CompletionItemKind,
  WorkspaceFolder,
  Range,
  FileCreateEvent,
  TextDocumentChangeEvent,
} from "vscode";
import { dirname, join, resolve } from "path";
import { existsSync } from "fs";
import { URI } from "vscode-uri";
import { SPItem, IncludeItem } from "./spItems";
import { events } from "../Misc/sourceEvents";
import {
  getLastFuncName,
  isInAComment,
  isFunction,
  getLastEnumStructNameOrMethodMap,
  isInAString,
} from "../Providers/spDefinitionProvider";
import { globalIdentifier } from "../Misc/spConstants";
import { FileItems } from "./spFilesRepository";
import {
  handleAddedDocument,
  handleDocumentChange,
  newDocumentCallback,
} from "./spFileHandlers";

export class ItemsRepository implements Disposable {
  public fileItems: Map<string, FileItems>;
  public documents: Set<string>;
  private globalState: Memento;

  constructor(globalState?: Memento) {
    this.fileItems = new Map<string, FileItems>();
    this.documents = new Set<string>();
    this.globalState = globalState;
  }

  public dispose() {}

  public handleAddedDocument(event: FileCreateEvent) {
    handleAddedDocument(this, event);
  }

  public handleDocumentChange(event: TextDocumentChangeEvent) {
    handleDocumentChange(this, event);
  }

  public handleNewDocument(document: TextDocument) {
    newDocumentCallback(this, document.uri);
  }

  public handleDocumentOpening(filePath: string) {
    newDocumentCallback(this, URI.file(filePath));
  }

  public getEventCompletions(): CompletionList {
    return new CompletionList(events);
  }

  getAllInheritances(variableType: string, allCompletions: SPItem[]): string[] {
    let methodMapItem = allCompletions.find(
      (e) => e.kind === CompletionItemKind.Class && e.name === variableType
    );
    if (methodMapItem === undefined || methodMapItem.parent === undefined) {
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
    lastEnumStructOrMethodMap: string
  ) {
    let i = position.character - 1;
    let bCounter = 0;
    let pCounter = 0;
    let isNameSpace = false;
    while (i >= 0) {
      if (/\w/.test(line[i])) {
        i--;
      } else if (line[i] === ".") {
        i--;
        break;
      } else if (line[i] === ":") {
        i--;
        if (line[i] === ":") {
          i--;
          isNameSpace = true;
          break;
        }
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
        } else if (line[i] === ":") {
          i--;
          if (line[i] === ":") {
            wordCounter++;
            words[wordCounter] = "";
            isNameSpace = true;
          }
        } else {
          break;
        }
      }
      i--;
    }
    let variableType: string;

    if (isNameSpace) {
      variableType = words[words.length - 1];
    } else {
      if (
        lastEnumStructOrMethodMap !== globalIdentifier &&
        words[words.length - 1] === "this"
      ) {
        variableType = lastEnumStructOrMethodMap;
      } else {
        variableType = allItems.find(
          (e) =>
            (e.kind === CompletionItemKind.Variable &&
              [globalIdentifier, lastFuncName].includes(e.parent) &&
              e.name === words[words.length - 1]) ||
            (e.kind === CompletionItemKind.Function &&
              e.name === words[words.length - 1]) ||
            (e.kind === CompletionItemKind.Class &&
              e.name === words[words.length - 1])
        ).type;
      }
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
    return { variableType, words };
  }

  getAllItems(uri: URI): SPItem[] {
    let workspaceFolder = Workspace.getWorkspaceFolder(uri);
    let includes = new Set<string>();
    let MainPath: string =
      Workspace.getConfiguration("sourcepawn", workspaceFolder).get(
        "MainPath"
      ) || "";
    let allItems;
    if (MainPath !== "") {
      if (!existsSync(MainPath)) {
        let workspace: WorkspaceFolder = Workspace.workspaceFolders[0];
        MainPath = join(workspace.uri.fsPath, MainPath);
        if (!existsSync(MainPath)) {
          throw new Error("MainPath is incorrect.");
        }
      }
      let uri = URI.file(MainPath).toString();
      allItems = this.fileItems.get(uri);
      if (!includes.has(uri)) {
        includes.add(uri);
      }
    } else {
      allItems = this.fileItems.get(uri.toString());
      includes.add(uri.toString());
    }
    if (allItems !== undefined) {
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
    let file_completions: FileItems = this.fileItems.get(file);
    let completion_list: SPItem[] = [];
    if (file_completions) {
      return Array.from(file_completions.values());
    }
    return completion_list;
  }

  getIncludedFiles(completions: FileItems, files: Set<string>) {
    for (let include of completions.includes) {
      if (!files.has(include.uri)) {
        files.add(include.uri);
        let include_completions = this.fileItems.get(include.uri);
        if (include_completions) {
          this.getIncludedFiles(include_completions, files);
        }
      }
    }
  }

  getItemFromPosition(document: TextDocument, position: Position): SPItem[] {
    let range = document.getWordRangeAtPosition(position);
    // First check if we are dealing with a method or property.
    let isMethod: boolean = false;
    let isConstructor: boolean = false;
    let match: RegExpMatchArray;
    let directoryPath = dirname(document.uri.fsPath);

    let word: string = document.getText(range);
    let allItems = this.getAllItems(document.uri);

    if (isInAComment(range, document.uri, allItems)) {
      return undefined;
    }

    // Check if include file
    let includeLine = document.lineAt(position.line).text;

    if (isInAString(range, includeLine)) {
      return undefined;
    }

    match = includeLine.match(/^\s*#include\s+<([A-Za-z0-9\-_\/.]+)>/);
    if (match === null) {
      match = includeLine.match(/^\s*#include\s+"([A-Za-z0-9\-_\/.]+)"/);
    }
    if (match !== null) {
      let file: string = match[1];
      let fileMatchLength = file.length;
      let fileStartPos = includeLine.search(file);
      // If no extension is provided, it's a .inc file
      if (!/.sp\s*$/g.test(file) && !/.inc\s*$/g.test(file)) {
        file += ".inc";
      }
      let defRange = new Range(
        position.line,
        fileStartPos,
        position.line,
        fileStartPos + fileMatchLength
      );
      let uri: string;
      let incFilePath;
      let smHome: string =
        Workspace.getConfiguration(
          "sourcepawn",
          Workspace.getWorkspaceFolder(document.uri)
        ).get("SourcemodHome") || "";
      let potentialIncludePaths = [
        directoryPath,
        join(directoryPath, "include/"),
        smHome,
      ];
      for (let includePath of potentialIncludePaths) {
        incFilePath = resolve(includePath, file);
        if (existsSync(resolve(includePath, file))) {
          break;
        }
      }
      for (let parsedUri of this.documents.values()) {
        if (parsedUri == URI.file(incFilePath).toString()) {
          uri = parsedUri;
          break;
        }
      }
      return [new IncludeItem(uri, defRange)];
    }
    if (range.start.character > 1) {
      let newPosStart = new Position(
        range.start.line,
        range.start.character - 2
      );
      let newPosEnd = new Position(range.start.line, range.start.character);
      let newRange = new Range(newPosStart, newPosEnd);
      let char = document.getText(newRange);
      isMethod = /(?:\w+\.|\:\:)/.test(char);
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

    let lastFunc: string = getLastFuncName(position, document, allItems);
    let {
      lastEnumStructOrMethodMap,
      isAMethodMap,
    } = getLastEnumStructNameOrMethodMap(position, document, allItems);
    // If we match a property or a method of an enum struct
    // but not a local scopped variable inside an enum struct's method.
    if (
      lastEnumStructOrMethodMap !== globalIdentifier &&
      lastFunc === globalIdentifier &&
      !isAMethodMap
    ) {
      let items = allItems.filter(
        (item) =>
          [
            CompletionItemKind.Method,
            CompletionItemKind.Property,
            CompletionItemKind.Constructor,
          ].includes(item.kind) &&
          item.parent === lastEnumStructOrMethodMap &&
          item.name === word
      );
      if (items.length !== 0) {
        return items;
      }
    }

    if (isMethod) {
      let line = document.lineAt(position.line).text;
      // If we are dealing with a method or property, look for the type of the variable
      let { variableType, words } = this.getTypeOfVariable(
        line,
        position,
        allItems,
        lastFunc,
        lastEnumStructOrMethodMap
      );
      // Get inheritances from methodmaps
      let variableTypes: string[] = this.getAllInheritances(
        variableType,
        allItems
      );
      // Find and return the matching item
      let items = allItems.filter(
        (item) =>
          [
            CompletionItemKind.Method,
            CompletionItemKind.Property,
            CompletionItemKind.Constructor,
          ].includes(item.kind) &&
          variableTypes.includes(item.parent) &&
          item.name === word
      );
      return items;
    }

    if (isConstructor) {
      let items = this.getAllItems(document.uri).filter(
        (item) =>
          item.kind === CompletionItemKind.Constructor && item.name === match[1]
      );
      return items;
    }
    // Check if we are dealing with a function
    let bIsFunction = isFunction(
      range,
      document,
      document.lineAt(position.line).text.length
    );
    let items = [];
    if (bIsFunction) {
      if (lastEnumStructOrMethodMap !== globalIdentifier) {
        // Check for functions and methods
        items = allItems.filter((item) => {
          if (
            [
              CompletionItemKind.Method,
              CompletionItemKind.Constructor,
            ].includes(item.kind) &&
            item.name === word &&
            item.parent === lastEnumStructOrMethodMap
          ) {
            return true;
          } else if (
            [
              CompletionItemKind.Function,
              CompletionItemKind.Interface,
            ].includes(item.kind) &&
            item.name === word
          ) {
            return true;
          }
          return false;
        });
        return items;
      } else {
        items = allItems.filter(
          (item) =>
            [
              CompletionItemKind.Function,
              CompletionItemKind.Interface,
            ].includes(item.kind) && item.name === word
        );
        return items;
      }
    }
    items = allItems.filter(
      (item) =>
        ![
          CompletionItemKind.Method,
          CompletionItemKind.Property,
          CompletionItemKind.Constructor,
          CompletionItemKind.Function,
        ].includes(item.kind) &&
        item.name === word &&
        item.parent === lastFunc
    );
    if (items.length > 0) {
      return items;
    }
    items = allItems.filter((item) => {
      if (
        [
          CompletionItemKind.Method,
          CompletionItemKind.Property,
          CompletionItemKind.Constructor,
        ].includes(item.kind)
      ) {
        return false;
      }
      if (item.parent !== undefined) {
        if (
          [CompletionItemKind.Class, CompletionItemKind.EnumMember].includes(
            item.kind
          )
        ) {
          return item.name === word;
        }
        if (item.enumStructName !== undefined) {
          return (
            item.parent === globalIdentifier &&
            item.name === word &&
            item.enumStructName === lastEnumStructOrMethodMap
          );
        }
        return item.parent === globalIdentifier && item.name === word;
      }
      return item.name === word;
    });
    return items;
  }
}
