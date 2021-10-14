import { ItemsRepository, FileItems } from "./spItemsRepository";
import {
  FunctionItem,
  MacroItem,
  DefineItem,
  EnumItem,
  EnumMemberItem,
  VariableItem,
  MethodItem,
  FunctionParam,
  PropertyItem,
  EnumStructItem,
  SPItem,
  MethodMapItem,
  TypeDefItem,
} from "./spItems";
import { isControlStatement } from "./spDefinitions";
import {
  CompletionItemKind,
  Location,
  Range,
  workspace as Workspace,
} from "vscode";
import { existsSync, readFileSync } from "fs";
import { basename, resolve, dirname } from "path";
import { URI } from "vscode-uri";
import { globalIdentifier } from "./spGlobalIdentifier";

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

enum State {
  None,
  EnumStruct,
  Enum,
  Methodmap,
  Property,
  Function,
  Loop,
  Macro,
}

class Parser {
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

  constructor(
    lines: string[],
    file: string,
    IsBuiltIn: boolean,
    completions: FileItems,
    itemsRepository: ItemsRepository
  ) {
    this.completions = completions;
    this.state = [State.None];
    this.lineNb = -1;
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
  }

  parse() {
    let line: string;
    line = this.lines[0];
    while (line !== undefined) {
      this.searchForDefinesInString(line);
      this.interpLine(line);
      line = this.lines.shift();
      this.lineNb++;
    }
  }

  interpLine(line: string) {
    // EOF
    if (line === undefined) return;
    // Match define
    let match = line.match(/\s*#define\s+(\w+)\s+([^]+)/);
    if (match) {
      this.read_define(match, line);
      // Re-read the line now that define has been added to the array.
      this.searchForDefinesInString(line);
      return;
    }

    match = line.match(/^\s*#define\s+(\w+)\s*\(([^\)]*)\)/);
    if (match) {
      this.readMacro(match, line);
      return;
    }

    // Match global include
    match = line.match(/^\s*#include\s+<([A-Za-z0-9\-_\/.]+)>/);
    if (match) {
      this.read_include(match);
      return;
    }

    // Match relative include
    match = line.match(/^\s*#include\s+"([A-Za-z0-9\-_\/.]+)"/);
    if (match) {
      this.read_include(match);
      return;
    }

    // Match enum structs
    match = line.match(/^\s*(?:enum\s+struct\s+)(\w*)\s*[^\{]*/);
    if (match) {
      this.read_enums(match, line, true);
      return;
    }
    // Match enums
    match = line.match(/^\s*(?:enum\s+)(\w*)\s*[^\{]*/);
    if (match) {
      this.read_enums(match, line, false);
      return;
    }

    // Match for loop iteration variable only in the current file
    match = line.match(/^\s*(?:for\s*\(\s*int\s+)([A-Za-z0-9_]*)/);
    if (match) {
      this.read_loop_variables(match, line);
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
      this.read_variables(match, line);
      return;
    }

    match = line.match(/\s*\/\*/);
    if (match) {
      this.scratch = [];
      this.consume_multiline_comment(line, false);
      return;
    }

    match = line.match(/^\s*\/\//);
    if (match) {
      this.scratch = [];
      this.consume_multiline_comment(line, true);
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
        this.read_property(match, line);
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

    match = line.match(/\s*typedef\s+(\w+)\s*\=\s*function\s+(\w+).*/);
    if (match) {
      this.readTypeDef(match, line);
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
      this.read_function(line);
    }

    match = line.match(/^\s*}/);
    if (match) {
      if (/^\s*\}\s*\belse\b\s*\{/.test(line)) {
        return;
      }
      let state = this.state[this.state.length - 1];
      if (state === State.Function && this.state_data !== undefined) {
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

  read_define(match: RegExpMatchArray, line: string): void {
    this.definesMap.set(match[1], this.file);
    let range = this.makeDefinitionRange(match[1], line);
    let fullRange = PositiveRange(this.lineNb, 0, line.length);
    this.completions.add(
      match[1],
      new DefineItem(
        match[1],
        match[2],
        this.file,
        range,
        this.IsBuiltIn,
        fullRange
      )
    );
    return;
  }

  readMacro(match: RegExpMatchArray, line: string): void {
    let { description, params } = this.parse_doc_comment();
    let nameMatch = match[1];
    let details = `${nameMatch}(${match[2]})`;
    let range = this.makeDefinitionRange(nameMatch, line);
    // Add the macro to the array of known macros
    this.macroArr.push(nameMatch);
    this.completions.add(
      nameMatch,
      new MacroItem(
        nameMatch,
        details,
        description,
        params,
        this.file,
        this.IsBuiltIn,
        range,
        "",
        undefined
      )
    );
  }

  read_include(match: RegExpMatchArray) {
    // Include guard to avoid extension crashs.
    if (IsIncludeSelfFile(this.file, match[1])) return;
    this.completions.resolve_import(match[1], this.documents, this.IsBuiltIn);
    return;
  }

  read_enums(match, line: string, IsStruct: boolean) {
    let { description, params } = this.parse_doc_comment();
    if (IsStruct) {
      // Create a completion for the enum struct itself if it has a name
      let enumStructName = match[1];
      let range = this.makeDefinitionRange(enumStructName, line);
      var enumStructCompletion: EnumStructItem = new EnumStructItem(
        enumStructName,
        this.file,
        description,
        range
      );
      this.completions.add(enumStructName, enumStructCompletion);
      this.state.push(State.EnumStruct);
      this.state_data = {
        name: enumStructName,
      };
      return;
    } else {
      let nameMatch = match[0].match(/^\s*(?:enum\s*)(\w*)/);
      if (nameMatch) {
        // Create a completion for the enum itself if it has a name
        let range = this.makeDefinitionRange(match[1], line);
        var enumCompletion: EnumItem = new EnumItem(
          nameMatch[1],
          this.file,
          description,
          range
        );
        this.completions.add(nameMatch[1], enumCompletion);
      } else {
        var enumCompletion: EnumItem = new EnumItem(
          "",
          this.file,
          description,
          undefined
        );
        this.completions.add("", enumCompletion);
      }

      // Set max number of iterations for safety
      let iter = 0;
      // Match all the enum members
      while (iter < 100 && !/\s*(\}\s*\;?)/.test(line)) {
        iter++;
        line = this.lines.shift();
        this.lineNb++;
        // Stop early if it's the end of the file
        if (line === undefined) {
          return;
        }
        match = line.match(/^\s*(\w*)\s*.*/);

        // Skip if didn't match
        if (!match) {
          continue;
        }
        let enumMemberName = match[1];
        // Try to match multiblock comments
        let enumMemberDescription: string;
        match = line.match(/\/\*\*<?\s*(.+?(?=\*\/))/);
        if (match) {
          enumMemberDescription = match[1];
        }
        match = line.match(/\/\/<?\s*(.*)/);
        if (match) {
          enumMemberDescription = match[1];
        }
        let range = this.makeDefinitionRange(enumMemberName, line);
        this.completions.add(
          enumMemberName,
          new EnumMemberItem(
            enumMemberName,
            this.file,
            enumMemberDescription,
            enumCompletion,
            range,
            this.IsBuiltIn
          )
        );
        this.searchForDefinesInString(line);
      }
      if (nameMatch) {
        this.addFullRange(nameMatch[1]);
      }
      return;
    }
  }

  read_loop_variables(match, line: string) {
    this.state.push(State.Loop);
    if (this.IsBuiltIn) {
      return;
    }
    this.AddVariableCompletion(match[1], line, "int");
    return;
  }

  read_variables(match, line: string) {
    let match_variables = [];
    let match_variable: RegExpExecArray;
    // Check if it's a multiline declaration
    let commentMatch = line.match(/\/\//);
    let croppedLine = line;
    if (commentMatch) {
      croppedLine = line.slice(0, commentMatch.index);
    }
    if (/(;)(?:\s*|)$/.test(croppedLine)) {
      // Separate potential multiple declarations
      let re = /\s*(?:(?:const|static|public|stock)\s+)*(\w*)\s*(?:\[(?:[A-Za-z_0-9+* ]*)\])*\s+(\w+)(?:\[(?:[A-Za-z_0-9+* ]*)\])*(?:\s*=\s*(?:(?:\"[^]*\")|(?:\'[^]*\')|(?:[^,]+)))?/g;
      while ((match_variable = re.exec(line)) != null) {
        match_variables.push(match_variable);
      }
      for (let variable of match_variables) {
        let variable_completion = variable[2].match(
          /(?:\s*)?([A-Za-z_,0-9]*)(?:(?:\s*)?(?:=(?:.*)))?/
        )[1];
        if (!this.IsBuiltIn) {
          this.AddVariableCompletion(variable_completion, line, variable[1]);
        }
      }
    } else {
      let parseLine: boolean = true;
      while (parseLine) {
        parseLine = !match[1].match(/(;)\s*$/);
        // Separate potential multiple declarations
        match_variables = match[1].match(
          /(?:\s*)?([A-Za-z0-9_\[`\]]+(?:\s+)?(?:\=(?:(?:\s+)?(?:[\(].*?[\)]|[\{].*?[\}]|[\"].*?[\"]|[\'].*?[\'])?(?:[A-Za-z0-9_\[`\]]*)))?(?:\s+)?|(!,))/g
        );
        if (!match_variables) {
          break;
        }
        for (let variable of match_variables) {
          let variable_completion = variable.match(
            /(?:\s*)?([A-Za-z_,0-9]*)(?:(?:\s*)?(?:=(?:.*)))?/
          )[1];
          if (!this.IsBuiltIn) {
            this.AddVariableCompletion(variable_completion, line, "");
          }
        }
        match[1] = this.lines.shift();
        line = match[1];
        this.lineNb++;
      }
    }
    return;
  }

  consume_multiline_comment(
    current_line: string,
    use_line_comment: boolean = false
  ) {
    let iter = 0;
    while (
      current_line !== undefined &&
      iter < 100 &&
      ((/^\s*\/\//.test(current_line) && use_line_comment) ||
        (!/\*\//.test(current_line) && !use_line_comment))
    ) {
      iter++;
      this.scratch.push(current_line.replace(/^\s*\/\//, ""));
      current_line = this.lines.shift();

      this.lineNb++;
    }
    // Removes the */ from the doc comment
    if (!use_line_comment) {
      current_line = this.lines.shift();
      this.lineNb++;
    }
    this.searchForDefinesInString(current_line);
    this.interpLine(current_line);
    return;
  }

  read_property(match, line) {
    let { description, params } = this.parse_doc_comment();
    let name_match: string = match[2];
    this.lastFuncName = name_match;
    let range = this.makeDefinitionRange(name_match, line);
    let NewPropertyCompletion = new PropertyItem(
      this.state_data.name,
      name_match,
      this.file,
      description,
      range,
      match[1]
    );
    this.completions.add(
      name_match + this.state_data.name,
      NewPropertyCompletion
    );
  }

  clean_param(partial_params_match: string) {
    let unused_comma = partial_params_match.match(/(\))(?:\s*)(?:;)?(?:\s*)$/);
    if (unused_comma) {
      partial_params_match = partial_params_match.replace(unused_comma[1], "");
    }
    return partial_params_match;
  }

  readTypeDef(match: RegExpMatchArray, line: string): void {
    let name = match[1];
    let type = match[2];
    let range = this.makeDefinitionRange(name, line);
    let { description, params } = this.parse_doc_comment();
    let fullRange = new Range(this.lineNb, 0, this.lineNb, line.length);
    this.completions.add(
      name,
      new TypeDefItem(
        name,
        match[0],
        this.file,
        description,
        type,
        range,
        fullRange
      )
    );
  }

  read_function(line: string) {
    if (line === undefined) {
      return;
    }
    let newSyntaxRe: RegExp = /^\s*(?:(?:stock|public|native|forward|static)\s+)*(?:(\w*(?:\s*\[[\w \+\-\*]*\]\s*)?)\s+)?(\w*)\s*\((.*(?:\)|,|{))?\s*/;
    let match: RegExpMatchArray = line.match(newSyntaxRe);
    if (!match) {
      match = line.match(
        /^\s*(?:(?:static|native|stock|public|forward)\s+)*(?:(\w+)\s*:)?\s*(\w*)\s*\(([^\)]*(?:\)?))(?:\s*)(?:\{?)(?:\s*)(?:[^\;\s]*);?\s*$/
      );
    }
    let isMethod: boolean =
      this.state.includes(State.Methodmap) ||
      this.state.includes(State.EnumStruct);

    // We can't declare a function inside a function, this is a call.
    // cancel the parsing
    if (this.state[this.state.length - 1] === State.Function) {
      return;
    }
    if (match) {
      let { description, params } = this.parse_doc_comment();
      let nameMatch = match[2];
      // Stop if it's a macro being called
      if (this.macroArr.length > 0) {
        let tmpStr = "";
        if (this.macroArr.length > 1) {
          tmpStr = `\\b(?:${this.macroArr.join("|")})\\b`;
        } else {
          tmpStr = `\\b(?:${this.macroArr[0]})\\b`;
        }
        let macroRe = new RegExp(tmpStr);
        if (macroRe.test(nameMatch)) {
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
          this.state.push(State.Macro);
          return;
        }
      }

      let lineMatch = this.lineNb;
      let type = match[1];
      let paramsMatch = match[3] === undefined ? "" : match[3];
      this.AddParamsDef(paramsMatch, nameMatch, line);
      // Iteration safety in case something goes wrong
      let maxiter = 0;
      let matchEndRegex: RegExp = /(\{|\;)\s*(?:(?:\/\/|\/\*)(?:.*))?$/;
      let isNativeOrForward = /\bnative\b|\bforward\b/.test(match[0]);
      let matchEnd = matchEndRegex.test(line);
      let pCount = getParenthesisCount(line);
      let matchLastParenthesis = pCount === 0;
      let range = this.makeDefinitionRange(nameMatch, line);

      while (
        !(matchLastParenthesis && matchEnd) &&
        line !== undefined &&
        maxiter < 20
      ) {
        maxiter++;
        line = this.lines.shift();
        this.lineNb++;
        if (!matchLastParenthesis) {
          this.AddParamsDef(line, nameMatch, line);
          this.searchForDefinesInString(line);
          paramsMatch += line;
          pCount += getParenthesisCount(line);
          matchLastParenthesis = pCount === 0;
        }
        if (!matchEnd) {
          if (matchLastParenthesis && /\,\s*$/.test(paramsMatch)) {
            // If the statement ends with a comma, we are in an array declaration
            return;
          }
          matchEnd = matchEndRegex.test(line);
        }
      }
      if (!matchEnd) {
        return;
      }
      let endSymbol = line.match(matchEndRegex);
      if (endSymbol === null) {
        return;
      }

      if (isNativeOrForward) {
        if (endSymbol[1] === "{") return;
      } else {
        if (endSymbol[1] === ";" || endSymbol[1] === ",") {
          return;
        } else if (!isSingleLineFunction(line)) {
          this.state.push(State.Function);
        }
      }
      this.lastFuncLine = lineMatch;
      this.lastFuncName = nameMatch;
      // Treat differently if the function is declared on multiple lines
      paramsMatch = /\)\s*(?:\{|;)?\s*$/.test(match[0])
        ? match[0]
        : match[0].replace(/\(.*\s*$/, "(") +
          paramsMatch.replace(/\s*\w+\s*\(\s*/g, "").replace(/\s+/gm, " ");
      if (params.length === 0) {
        params = getParamsFromDeclaration(paramsMatch);
      }
      if (isMethod) {
        let fullRange: Range;
        if (isNativeOrForward) {
          let end = range.start.line === this.lineNb ? line.length : 0;
          fullRange = new Range(range.start.line, 0, this.lineNb, end);
        }
        this.completions.add(
          nameMatch + this.state_data.name,
          new MethodItem(
            this.state_data.name,
            nameMatch,
            paramsMatch.replace(/;\s*$/g, "").replace(/{\s*$/g, "").trim(),
            description,
            params,
            type,
            this.file,
            range,
            this.IsBuiltIn,
            fullRange
          )
        );
        return;
      }
      // For small files, the parsing is too fast and functions get overwritten by their own calls.
      // If we define a function somewhere, we won't redefine it elsewhere. We can safely ignore it.
      if (this.completions.get(nameMatch)) {
        return;
      }
      let fullRange: Range;
      if (isNativeOrForward) {
        let end = range.start.line === this.lineNb ? line.length : 0;
        fullRange = new Range(range.start.line, 0, this.lineNb, end);
      }
      this.completions.add(
        nameMatch,
        new FunctionItem(
          nameMatch,
          paramsMatch.replace(/;\s*$/g, "").replace(/{\s*$/g, "").trim(),
          description,
          params,
          this.file,
          this.IsBuiltIn,
          range,
          type,
          fullRange
        )
      );
    }
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
        new PropertyItem(this.state_data.name, name, this.file, "", range, type)
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
    var range = PositiveRange(this.lineNb, start, end);
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

  searchForDefinesInString(line: string): void {
    if (line === undefined) {
      return;
    }
    let isBlockComment = false;
    let isDoubleQuoteString = false;
    let isSingleQuoteString = false;
    let matchDefine: RegExpExecArray;
    const re: RegExp = /(?:"|'|\/\/|\/\*|\*\/|\w+)/g;
    let defineFile: string;
    while ((matchDefine = re.exec(line))) {
      if (matchDefine[0] === '"' && !isSingleQuoteString) {
        isDoubleQuoteString = !isDoubleQuoteString;
      } else if (matchDefine[0] === "'" && !isDoubleQuoteString) {
        isSingleQuoteString = !isSingleQuoteString;
      } else if (
        matchDefine[0] === "//" &&
        !isDoubleQuoteString &&
        !isSingleQuoteString
      ) {
        break;
      } else if (
        matchDefine[0] === "/*" ||
        (matchDefine[0] === "*/" &&
          !isDoubleQuoteString &&
          !isSingleQuoteString)
      ) {
        isBlockComment = !isBlockComment;
      }
      if (isBlockComment || isDoubleQuoteString || isSingleQuoteString) {
        continue;
      }
      defineFile =
        this.definesMap.get(matchDefine[0]) ||
        this.enumMemberMap.get(matchDefine[0]);
      if (defineFile !== undefined) {
        let range = PositiveRange(
          this.lineNb,
          matchDefine.index,
          matchDefine.index + matchDefine[0].length
        );
        let location = new Location(URI.file(this.file), range);
        // Treat defines from the current file differently or they will get
        // overwritten at the end of the parsing.
        if (defineFile === this.file) {
          let define = this.completions.get(matchDefine[0]);
          if (define === undefined) {
            continue;
          }
          define.calls.push(location);
          this.completions.add(matchDefine[0], define);
          continue;
        }
        defineFile = defineFile.startsWith("file://")
          ? defineFile
          : URI.file(defineFile).toString();
        let items = this.itemsRepository.completions.get(defineFile);
        if (items === undefined) {
          continue;
        }
        let define = items.get(matchDefine[0]);
        if (define === undefined) {
          continue;
        }
        define.calls.push(location);
        items.add(matchDefine[0], define);
      }
    }
    return;
  }

  getAllMembers(
    items: SPItem[],
    kind: CompletionItemKind
  ): Map<string, string> {
    if (items == undefined) {
      return new Map();
    }
    let defines = new Map();
    let smHome =
      Workspace.getConfiguration("sourcepawn").get<string>("SourcemodHome") ||
      "";
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

function purgeCalls(item: SPItem, file: string): void {
  let uri = URI.file(file);
  if (item.calls === undefined) return;
  item.calls = item.calls.filter((e) => {
    uri === e.uri;
  });
}

function PositiveRange(
  lineNb: number,
  start: number = 0,
  end: number = 0
): Range {
  lineNb = lineNb > 0 ? lineNb : 0;
  start = start > 0 ? start : 0;
  end = end > 0 ? end : 0;
  return new Range(lineNb, start, lineNb, end);
}

function IsIncludeSelfFile(file: string, include: string): boolean {
  let baseName: string = basename(file);
  let match = include.match(/(\w*)(?:.sp|.inc)?$/);
  if (match) {
    return baseName == match[1];
  }
  return false;
}

export function getParamsFromDeclaration(decl: string): FunctionParam[] {
  let match = decl.match(/\((.+)\)/);
  if (!match) {
    return [];
  }
  // Remove the leading and trailing parenthesis
  decl = match[1] + ",";
  let params: FunctionParam[] = [];
  let re = /\s*(?:(?:const|static)\s+)?(?:(\w+)(?:\s*(?:\[(?:[A-Za-z_0-9+* ]*)\])?\s+|\s*\:\s*))?(\w+)(?:\[(?:[A-Za-z_0-9+* ]*)\])?(?:\s*=\s*(?:[^,]+))?/g;
  let matchVariable;
  while ((matchVariable = re.exec(decl)) != null) {
    params.push({ label: matchVariable[2], documentation: "" });
  }
  return params;
}

function isSingleLineFunction(line: string) {
  return /\{.*\}\s*$/.test(line);
}

function parentCounter(line: string): number {
  let counter = 0;
  if (line == null) {
    return 0;
  }
  for (let char of line) {
    if (char === "(") {
      counter++;
    } else if (char === ")") {
      counter--;
    }
  }
  return counter;
}

function getParenthesisCount(line: string): number {
  let pCount = 0;
  let inAString = false;
  for (let i = 0; i < line.length; i++) {
    let char = line[i];
    if (char === "'" || char === '"') {
      inAString = !inAString;
    } else if (!inAString && char === "(") {
      pCount++;
    } else if (!inAString && char === ")") {
      pCount--;
    }
  }
  return pCount;
}
