import { ItemsRepository, FileItems } from "../Providers/spItemsRepository";
import {
  VariableItem,
  FunctionParam,
  PropertyItem,
  SPItem,
  MethodMapItem,
  CommentItem,
} from "../Providers/spItems";
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

import { isControlStatement } from "../Providers/spDefinitions";
import { CompletionItemKind, Range, workspace as Workspace } from "vscode";
import { existsSync, readFileSync } from "fs";
import { resolve, dirname } from "path";
import { URI } from "vscode-uri";
import { globalIdentifier } from "../Providers/spGlobalIdentifier";
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
    let items = itemsRepository.getAllItems(URI.file(this.file).toString());
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
      // Re-read the line now that define has been added to the array.
      searchForDefinesInString(this, line);
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
      if (
        /^\s*(if|else|while|do|return|break|continue|delete|forward|native|property|enum|funcenum|functag|methodmap|struct|typedef|typeset|this|view_as|sizeof)/.test(
          line
        )
      )
        return;
      if (/^\s*public\s+native/.test(line)) return;
      readVariable(this, match, line);
      return;
    }

    match = line.match(/^\s*\/\*/);
    if (match) {
      this.scratch = [];
      consumeComment(this, line, false);
      return;
    }

    match = line.match(/^\s*\/\//);
    if (match) {
      this.scratch = [];
      consumeComment(this, line, true);
      return;
    }

    match = line.match(
      /^\s*methodmap\s+([a-zA-Z][a-zA-Z0-9_]*)(?:\s*<\s*([a-zA-Z][a-zA-Z0-9_]*))?/
    );
    if (match) {
      this.state.push(State.Methodmap);
      this.state_data = {
        name: match[1],
      };
      let { description, params } = this.parse_doc_comment();
      let range = this.makeDefinitionRange(match[1], line);
      var methodMapCompletion = new MethodMapItem(
        match[1],
        match[2],
        line.trim(),
        description,
        this.file,
        range,
        this.IsBuiltIn
      );
      this.completions.add(match[1], methodMapCompletion);
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
      if (isControlStatement(line) || /\bfunction\b/.test(match[1])) {
        return;
      }
      if (this.state.includes(State.Property)) {
        if (!/;\s*$/.test(line)) {
          this.state.push(State.Function);
        }
        return;
      }
      readFunction(this, line);
    }

    match = line.match(/^\s*}/);
    if (match) {
      if (/^\s*\}\s*\belse\b\s*\{/.test(line)) {
        return;
      }
      let state = this.state[this.state.length - 1];
      if (state === State.None) {
      } else if (state === State.Function && this.state_data !== undefined) {
        // We are in a method
        this.lastFuncLine = 0;
        this.addFullRange(this.lastFuncName + this.state_data.name);
      } else if (state === State.Methodmap && this.state_data !== undefined) {
        // We are in a methodmap
        this.addFullRange(this.state_data.name);
        this.state_data = undefined;
      } else if (state === State.EnumStruct && this.state_data !== undefined) {
        // We are in an enum struct
        this.addFullRange(this.state_data.name);
        this.state_data = undefined;
      } else if (state === State.Property && this.state_data !== undefined) {
        // We are in a property
        this.addFullRange(this.lastFuncName + this.state_data.name);
      } else if (
        ![
          State.Methodmap,
          State.EnumStruct,
          State.Property,
          State.Loop,
          State.Macro,
        ].includes(state)
      ) {
        // We are in a regular function
        this.addFullRange(this.lastFuncName);
      }
      this.state.pop();
      return;
    }

    // Reset the comments buffer
    this.scratch = [];
    return;
  }

  clean_param(partial_params_match: string) {
    let unused_comma = partial_params_match.match(/(\))(?:\s*)(?:;)?(?:\s*)$/);
    if (unused_comma) {
      partial_params_match = partial_params_match.replace(unused_comma[1], "");
    }
    return partial_params_match;
  }

  parse_doc_comment(): {
    description: string;
    params: FunctionParam[];
  } {
    if (this.scratch === undefined) {
      let description = "";
      let params = [];
      return { description, params };
    }
    let description = (() => {
      let lines = [];
      for (let line of this.scratch) {
        if (/^\s*\/\*\*\s*/.test(line)) {
          //Check if @return or @error
          continue;
        }

        lines.push(
          line.replace(/^\s*\*\s+/, "\n").replace(/^\s*\/\/\s+/, "\n")
        );
      }
      return lines.join(" ");
    })();

    const paramRegex = /@param\s+([\w\.]+)\s+(.*)/;
    let params = (() => {
      let params = [];
      let currentParam;
      for (let line of this.scratch) {
        let match = line.match(paramRegex);
        if (match) {
          // Check if we already have a param description in the buffer.
          // If yes, save it.
          if (currentParam) {
            currentParam.documentation = currentParam.documentation.join(" ");
            params.push(currentParam);
          }
          currentParam = { label: match[1], documentation: [match[2]] };
        } else {
          // Check if it's a return or error description.
          if (/@(?:return|error)/.test(line)) {
            // Check if we already have a param description in the buffer.
            // If yes, save it.
            if (currentParam != undefined) {
              currentParam.documentation = currentParam.documentation.join(" ");
              params.push(currentParam);
              currentParam = undefined;
            }
          } else {
            // Check if we already have a param description in the buffer.
            // If yes, append the new line to it.
            let match = line.match(/\s*(?:\*|\/\/)\s*(.*)/);
            if (match && currentParam) {
              currentParam.documentation.push(match[1]);
            }
          }
        }
      }
      // Add the last param
      if (currentParam != undefined) {
        currentParam.documentation = currentParam.documentation.join(" ");
        params.push(currentParam);
      }

      return params;
    })();

    // Reset the comments buffer
    this.scratch = [];
    return { description, params };
  }

  AddVariableCompletion(
    name: string,
    line: string,
    type: string,
    funcName: string = undefined,
    isParamDef = false
  ): void {
    if (line === undefined) {
      return;
    }
    let range = this.makeDefinitionRange(name, line);
    let scope: string = globalIdentifier;
    let enumStructName: string;
    if (this.state.includes(State.EnumStruct)) {
      enumStructName = this.state_data.name;
    }
    if (this.lastFuncLine !== 0) {
      scope = this.lastFuncName;
    }
    if (funcName !== undefined) {
      scope = funcName;
    }
    // Custom key name for the map so the definitions don't override each others
    let mapName = name + scope + enumStructName;
    if (
      (this.state.includes(State.EnumStruct) ||
        this.state.includes(State.Methodmap)) &&
      (this.state.includes(State.Function) || isParamDef)
    ) {
      this.completions.add(
        mapName + this.lastFuncName,
        new VariableItem(name, this.file, scope, range, type, enumStructName)
      );
    } else if (this.state.includes(State.EnumStruct)) {
      this.completions.add(
        mapName,
        new PropertyItem(
          this.state_data.name,
          name,
          this.file,
          line,
          "",
          range,
          type
        )
      );
    } else {
      this.completions.add(
        mapName,
        new VariableItem(name, this.file, scope, range, type, globalIdentifier)
      );
    }
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

  AddParamsDef(params: string, funcName: string, line: string) {
    let match_variable: RegExpExecArray;
    let match_variables: RegExpExecArray[] = [];
    let re = /\s*(?:(?:const|static)\s+)?(?:(\w+)(?:\s*(?:\[(?:[A-Za-z_0-9+* ]*)\])?\s+|\s*\:\s*))?(\w+)(?:\[(?:[A-Za-z_0-9+* ]*)\])?(?:\s*=\s*(?:[^,]+))?/g;
    while ((match_variable = re.exec(params)) != null) {
      match_variables.push(match_variable);
    }
    for (let variable of match_variables) {
      let variable_completion = variable[2].match(
        /(?:\s*)?([A-Za-z_,0-9]*)(?:(?:\s*)?(?:=(?:.*)))?/
      )[1];
      if (!this.IsBuiltIn) {
        this.AddVariableCompletion(
          variable_completion,
          line,
          variable[1],
          funcName,
          true
        );
      }
    }
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
        let file = item.file;
        if (item.IsBuiltIn) {
          file = file.replace(smHome, "file://__sourcemod_builtin");
        }
        defines.set(item.name, file);
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

  addFullRange(key: string) {
    let completion = this.completions.get(key);
    if (completion && completion.fullRange === undefined) {
      let range = completion.range;
      let fullRange = new Range(
        range.start.line,
        range.start.character,
        this.lineNb,
        1
      );
      completion.fullRange = fullRange;
      this.completions.add(key, completion);
    }
  }
}
