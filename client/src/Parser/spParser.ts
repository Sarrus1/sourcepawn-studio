import { ItemsRepository } from "../Backend/spItemsRepository";
import { FileItems } from "../Backend/spFilesRepository";
import { SPItem, CommentItem } from "../Backend/spItems";
import { State } from "./stateEnum";
import { readDefine } from "./readDefine";
import { readMacro } from "./readMacro";
import { readInclude } from "./readInclude";
import { readEnum } from "./readEnum";
import { readLoopVariable } from "./readLoopVariable";
import { readVariable } from "./readVariable";
import { readProperty } from "./readProperty";
import { readTypeDef } from "./readTypeDef";
import { readTypeSet } from "./readTypeSet";
import { readFunction } from "./readFunction";
import { consumeComment } from "./consumeComment";
import { searchForDefinesInString } from "./searchForDefinesInString";
import { readMethodMap } from "./readMethodMap";
import { manageState } from "./manageState";

import { CompletionItemKind, Range, workspace as Workspace } from "vscode";
import { existsSync, readFileSync } from "fs";
import { resolve, dirname } from "path";
import { URI } from "vscode-uri";
import { purgeCalls, positiveRange, parentCounter } from "./utils";

export function parseFile(
  file: string,
  completions: FileItems,
  itemsRepository: ItemsRepository,
  IsBuiltIn: boolean = false
) {
  if (!existsSync(file)) {
    return;
  }
  let data = readFileSync(file, "utf-8");

  // Test for symbolic links
  let match = data.match(/^(?:\.\.\/)+(?:[\/\w\-])+\.\w+/);
  if (match !== null) {
    let folderpath = dirname(file);
    file = resolve(folderpath, match[0]);
    data = readFileSync(file, "utf-8");
  }
  parseText(data, file, completions, itemsRepository, IsBuiltIn);
}

export function parseText(
  data: string,
  file: string,
  completions: FileItems,
  itemsRepository: ItemsRepository,
  IsBuiltIn: boolean = false
) {
  if (data === undefined) {
    return; // Asked to parse empty file
  }
  let lines = data.split("\n");
  let parser = new Parser(lines, file, IsBuiltIn, completions, itemsRepository);
  parser.parse();
}

export class Parser {
  completions: FileItems;
  state: State[];
  scratch: any;
  state_data: any;
  lines: string[];
  lineNb: number;
  file: string;
  IsBuiltIn: boolean;
  documents: Set<string>;
  lastFuncLine: number;
  lastFuncName: string;
  definesMap: Map<string, string>;
  enumMemberMap: Map<string, string>;
  macroArr: string[];
  itemsRepository: ItemsRepository;
  debugging: boolean;
  anonymousEnumCount: number;

  constructor(
    lines: string[],
    file: string,
    IsBuiltIn: boolean,
    completions: FileItems,
    itemsRepository: ItemsRepository
  ) {
    this.completions = completions;
    this.state = [State.None];
    this.lineNb = 0;
    this.lines = lines;
    this.file = file;
    this.IsBuiltIn = IsBuiltIn;
    this.documents = itemsRepository.documents;
    this.lastFuncLine = 0;
    this.lastFuncName = "";
    // Get all the items from the itemsRepository for this file
    let items = itemsRepository.getAllItems(URI.file(this.file));
    this.definesMap = this.getAllMembers(items, CompletionItemKind.Constant);
    this.enumMemberMap = this.getAllMembers(
      items,
      CompletionItemKind.EnumMember
    );
    this.macroArr = this.getAllMacros(items);
    this.itemsRepository = itemsRepository;
    let debugSetting = Workspace.getConfiguration("sourcepawn").get(
      "trace.server"
    );
    this.debugging = debugSetting == "messages" || debugSetting == "verbose";
    this.anonymousEnumCount = 0;
  }

  parse() {
    let line: string;
    line = this.lines.shift();
    while (line !== undefined) {
      searchForDefinesInString(this, line);
      this.interpLine(line);
      line = this.lines.shift();
      this.lineNb++;
    }
  }

  interpLine(line: string) {
    // EOF
    if (line === undefined) return;

    // Match trailing single line comments
    let match = line.match(/^\s*[^\/\/\s]+(\/\/.+)$/);
    if (match) {
      let lineNb = this.lineNb < 1 ? 0 : this.lineNb;
      let start: number = line.search(/\/\//);
      let range = new Range(lineNb, start, lineNb, line.length);
      this.completions.add(
        `comment${lineNb}--${Math.random()}`,
        new CommentItem(this.file, range)
      );
    }

    // Match trailing block comments
    match = line.match(/^\s*[^\/\*\s]+(\/\*.+)\*\//);
    if (match) {
      let lineNb = this.lineNb < 1 ? 0 : this.lineNb;
      let start: number = line.search(/\/\*/);
      let end: number = line.search(/\*\//);
      let range = new Range(lineNb, start, lineNb, end);
      this.completions.add(
        `comment${lineNb}--${Math.random()}`,
        new CommentItem(this.file, range)
      );
    }

    // Match define
    match = line.match(/^\s*#define\s+(\w+)\s+([^]+)/);
    if (match) {
      readDefine(this, match, line);
      return;
    }

    match = line.match(/^\s*#define\s+(\w+)\s*\(([^\)]*)\)/);
    if (match) {
      readMacro(this, match, line);
      return;
    }

    // Match global include
    match = line.match(/^\s*#include\s+<([A-Za-z0-9\-_\/.]+)>/);
    if (match) {
      readInclude(this, match);
      return;
    }

    // Match relative include
    match = line.match(/^\s*#include\s+"([A-Za-z0-9\-_\/.]+)"/);
    if (match) {
      readInclude(this, match);
      return;
    }

    // Match enum structs
    match = line.match(/^\s*(?:enum\s+struct\s+)(\w*)\s*[^\{]*/);
    if (match) {
      readEnum(this, match, line, true);
      return;
    }
    // Match enums
    match = line.match(/^\s*enum(?:\s+(\w+))?\s*[^\{]*/);
    if (match && !/;\s*$/.test(line)) {
      readEnum(this, match, line, false);
      return;
    }

    // Match for loop iteration variable only in the current file
    match = line.match(/^\s*(?:for\s*\(\s*int\s+)([A-Za-z0-9_]*)/);
    if (match) {
      readLoopVariable(this, match, line);
      return;
    }

    match = line.match(/^\s*typedef\s+(\w+)\s*\=\s*function\s+(\w+).*/);
    if (match) {
      readTypeDef(this, match, line);
      return;
    }

    match = line.match(/^\s*typeset\s+(\w+)/);
    if (match) {
      readTypeSet(this, match, line);
      return;
    }

    // Match variables only in the current file
    match = line.match(
      /^\s*(?:(?:new|static|const|decl|public|stock)\s+)*\w+(?:\[\])?\s+(\w+\s*(?:\[[A-Za-z0-9 +\-\*_]*\])*\s*(?:=\s*[^;,]+)?(?:,|;))/
    );
    if (match && !this.IsBuiltIn) {
      readVariable(this, match, line);
      return;
    }

    match = line.match(/^\s*\/\*/);
    if (match) {
      consumeComment(this, line, false);
      return;
    }

    match = line.match(/^\s*\/\//);
    if (match) {
      consumeComment(this, line, true);
      return;
    }

    match = line.match(
      /^\s*methodmap\s+([a-zA-Z][a-zA-Z0-9_]*)(?:\s*<\s*([a-zA-Z][a-zA-Z0-9_]*))?/
    );
    if (match) {
      readMethodMap(this, match, line);
      return;
    }

    // Match properties
    match = line.match(/^\s*property\s+([a-zA-Z]\w*)\s+([a-zA-Z]\w*)/);
    if (match) {
      if (this.state.includes(State.Methodmap)) {
        this.state.push(State.Property);
      }
      try {
        readProperty(this, match, line);
      } catch (e) {
        console.error(e);
        if (this.debugging) {
          console.error(`At line ${this.lineNb} of ${this.file}`);
        }
      }
      return;
    }

    match = line.match(
      /^\s*(\bwhile\b|\belse\b|\bif\b|\bswitch\b|\bcase\b|\bdo\b)/
    );
    if (match) {
      // Check if we are still in the conditionnal of the control statement
      // for example, an if statement's conditionnal can span over several lines
      // and call functions
      let parenthesisNB = parentCounter(line);
      let lineCounter = 0;
      let iter = 0;
      while (parenthesisNB !== 0 && iter < 100) {
        iter++;
        line = this.lines[lineCounter];
        lineCounter++;
        parenthesisNB += parentCounter(line);
      }
      // Now we test if the statement uses brackets, as short code blocks are usually
      // implemented without them.
      if (!/\{\s*$/.test(line)) {
        // Test the next line if we didn't match
        if (!/^\s*\{/.test(this.lines[lineCounter])) {
          return;
        }
      }
      this.state.push(State.Loop);
      return;
    }

    match = line.match(
      /^\s*(?:(?:static|native|stock|public|forward)\s+)*(?:[a-zA-Z\-_0-9]:)?([^\s]+)\s*(\w*)\s*\(([^\)]*(?:\)?))(?:\s*)(?:\{?)(?:\s*)(?:[^\;\s]*);?\s*$/
    );
    if (match) {
      readFunction(this, match, line);
    }

    match = line.match(/^\s*}/);
    if (match) {
      manageState(this, line);
      return;
    }

    // Reset the comments buffer
    this.scratch = [];
    return;
  }

  makeDefinitionRange(
    name: string,
    line: string,
    search: boolean = true
  ): Range {
    let re: RegExp = new RegExp(`\\b${name}\\b`);
    let start: number = search ? line.search(re) : 0;
    let end: number = search ? start + name.length : 0;
    var range = positiveRange(this.lineNb, start, end);
    return range;
  }

  getAllMembers(
    items: SPItem[],
    kind: CompletionItemKind
  ): Map<string, string> {
    if (items == undefined) {
      return new Map();
    }
    let defines = new Map();
    let workspaceFolder = Workspace.getWorkspaceFolder(URI.file(this.file));
    let smHome =
      Workspace.getConfiguration("sourcepawn", workspaceFolder).get<string>(
        "SourcemodHome"
      ) || "";
    // Replace \ escaping in Windows
    smHome = smHome.replace(/\\/g, "/");
    if (smHome === "") {
      return new Map();
    }
    for (let item of items) {
      if (item.kind === kind) {
        purgeCalls(item, this.file);
        defines.set(item.name, item.file);
      }
    }
    return defines;
  }

  getAllMacros(items: SPItem[]): string[] {
    if (items == undefined) {
      return [];
    }
    let arr: string[] = [];
    for (let e of items) {
      if (e.kind === CompletionItemKind.Interface) {
        arr.push(e.name);
      }
    }
    return arr;
  }
}
