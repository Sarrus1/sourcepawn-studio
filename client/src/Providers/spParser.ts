import { ItemsRepository, FileItems } from "./spItemsRepository";
import {
  FunctionItem,
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
} from "./spItems";
import { isControlStatement } from "./spDefinitions";
import {
  CompletionItemKind,
  Location,
  Range,
  workspace as Workspace,
} from "vscode";
import { existsSync, readFileSync } from "fs";
import { basename } from "path";
import { URI } from "vscode-uri";
import { isRegExp } from "util";

export function parseFile(
  file: string,
  completions: FileItems,
  itemsRepository: ItemsRepository,
  IsBuiltIn: boolean = false
) {
  if (!existsSync(file)) return;
  let data = readFileSync(file, "utf-8");
  parseText(data, file, completions, itemsRepository, IsBuiltIn);
}

export function parseText(
  data: string,
  file: string,
  completions: FileItems,
  itemsRepository: ItemsRepository,
  IsBuiltIn: boolean = false
) {
  if (typeof data === "undefined") {
    return; // Asked to parse empty file
  }
  let lines = data.split("\n");
  let parser = new Parser(lines, file, IsBuiltIn, completions, itemsRepository);
  parser.parse();
}

enum State {
  None,
  DocComment,
  EnumStruct,
  Methodmap,
  Property,
  Function,
  Loop,
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
  documents: Map<string, string>;
  lastFuncLine: number;
  lastFuncName: string;
  definesMap: Map<string, string>;
  enumMemberMap: Map<string, string>;
  itemsRepository: ItemsRepository;

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
    this.definesMap = this.getAllMembers(
      itemsRepository,
      CompletionItemKind.Constant
    );
    this.enumMemberMap = this.getAllMembers(
      itemsRepository,
      CompletionItemKind.EnumMember
    );
    this.itemsRepository = itemsRepository;
  }

  parse() {
    let line: string;
    line = this.lines[0];
    while (typeof line != "undefined") {
      this.searchForDefinesInString(line);
      this.interpLine(line);
      line = this.lines.shift();
      this.lineNb++;
    }
  }

  interpLine(line: string) {
    // EOF
    if (typeof line === "undefined") return;
    // Match define
    let match = line.match(/\s*#define\s+(\w+)\s+([^]+)/);
    if (match) {
      this.read_define(match, line);
      // Re-read the line now that define has been added to the array.
      this.searchForDefinesInString(line);
      return;
    }

    // Match global include
    match = line.match(/^\s*#include\s+<([A-Za-z0-9\-_\/.]+)>\s*$/);
    if (match) {
      this.read_include(match);
      return;
    }

    // Match relative include
    match = line.match(/^\s*#include\s+"([A-Za-z0-9\-_\/.]+)"\s*$/);
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
    if (match && !this.IsBuiltIn) {
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
      this.read_property(match, line);
      return;
    }

    match = line.match(/^\s*(\bwhile\b|\belse\b|\bif\b|\bswitch\b|\bcase\b)/);
    if (match) {
      this.state.push(State.Loop);
      return;
    }

    match = line.match(/}/);
    if (match) {
      if (this.state[this.state.length - 1] === State.Function) {
        this.lastFuncLine = 0;
      }
      this.state.pop();
      return;
    }

    match = line.match(
      /(?:(?:static|native|stock|public|forward)\s+)*(?:[a-zA-Z\-_0-9]:)?([^\s]+)\s*([A-Za-z_]*)\s*\(([^\)]*(?:\)?))(?:\s*)(?:\{?)(?:\s*)(?:[^\;\s]*);?\s*$/
    );
    if (match) {
      if (
        isControlStatement(line) ||
        this.state.includes(State.Property) ||
        /\bfunction\b/.test(match[1])
      ) {
        return;
      }
      this.read_function(line);
    }
    return;
  }

  read_define(match, line: string) {
    this.definesMap.set(match[1], this.file);
    let range = this.makeDefinitionRange(match[1], line);
    this.completions.add(
      match[1],
      new DefineItem(match[1], match[2], this.file, range, this.IsBuiltIn)
    );
    return;
  }

  read_include(match) {
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
        if (typeof line === "undefined") {
          return;
        }
        this.searchForDefinesInString(line);
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
      }
      return;
    }
  }

  read_loop_variables(match, line: string) {
    this.state.push(State.Loop);
    this.AddVariableCompletion(match[1], line, "int");
    return;
  }

  read_variables(match, line: string) {
    let match_variables = [];
    let match_variable: RegExpExecArray;
    // Check if it's a multiline declaration
    if (/(;)(?:\s*|)$/.test(line)) {
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
          this.AddVariableCompletion(
            variable_completion,
            line,
            variable[1],
            true
          );
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
            this.AddVariableCompletion(variable_completion, line, "", true);
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
      typeof current_line != "undefined" &&
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
    let range = this.makeDefinitionRange(name_match, line);
    let NewPropertyCompletion = new PropertyItem(
      this.state_data.name,
      name_match,
      this.file,
      description,
      range,
      match[1]
    );
    this.completions.add(name_match, NewPropertyCompletion);
  }

  clean_param(partial_params_match: string) {
    let unused_comma = partial_params_match.match(/(\))(?:\s*)(?:;)?(?:\s*)$/);
    if (unused_comma) {
      partial_params_match = partial_params_match.replace(unused_comma[1], "");
    }
    return partial_params_match;
  }

  read_function(line: string) {
    if (typeof line === "undefined") {
      return;
    }
    let newSyntaxRe: RegExp = /^\s*(?:(?:stock|public|native|forward|static)\s+)*(?:(\w*)\s+)?(\w*)\s*\((.*(?:\)|,|{))\s*/;
    let match: RegExpMatchArray = line.match(newSyntaxRe);
    if (!match) {
      match = line.match(
        /^\s*(?:(?:forward|static|native)\s+)+(\w*\s*:\s*|\w*\s+)?(\w*)\s*\(([^]*)(?:,|;)?\s*$/
      );
    }
    let isMethod: boolean =
      this.state.includes(State.Methodmap) ||
      this.state.includes(State.EnumStruct);
    if (match) {
      let { description, params } = this.parse_doc_comment();
      let nameMatch = match[2];
      let lineMatch = this.lineNb;
      let type = match[1];
      let paramsMatch = match[3];
      if (this.state.includes(State.EnumStruct)) {
        this.state.push(State.Function);
      }
      this.AddParamsDef(paramsMatch, nameMatch, line);
      // Iteration safety in case something goes wrong
      let maxiter = 0;
      let matchEndRegex: RegExp = /(\{|\;)/;
      let isNativeOrForward = /\bnative\b|\bforward\b/.test(match[0]);
      let matchEnd = matchEndRegex.test(line);
      let matchLastParenthesis = /\)/.test(paramsMatch);
      let range = this.makeDefinitionRange(nameMatch, line);
      while (
        !(matchLastParenthesis && matchEnd) &&
        typeof line != "undefined" &&
        maxiter < 20
      ) {
        maxiter++;
        line = this.lines.shift();
        this.lineNb++;
        if (!matchLastParenthesis) {
          matchLastParenthesis = /\)/.test(paramsMatch);
          this.AddParamsDef(line, nameMatch, line);
          this.searchForDefinesInString(line);
          paramsMatch += line;
        }
        if (!matchEnd) {
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
        if (endSymbol[0] === "{") return;
      } else {
        if (endSymbol[0] === ";") return;
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
            this.IsBuiltIn
          )
        );
        return;
      }
      // For small files, the parsing is too fast and functions get overwritten by their own calls.
      // If we define a function somewhere, we won't redefine it elsewhere. We can safely ignore it.
      if (this.completions.get(nameMatch)) {
        return;
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
          type
        )
      );
    }
  }

  parse_doc_comment(): {
    description: string;
    params: FunctionParam[];
  } {
    if (typeof this.scratch == "undefined") {
      let description = "";
      let params = [];
      return { description, params };
    }
    let description = (() => {
      let lines = [];
      for (let line of this.scratch) {
        //Check if @return or @error
        if (/^\s*\/\*\*\s*/.test(line)) {
          continue;
        }

        lines.push(
          line.replace(/^\s*\*\s+/, "\n").replace(/^\s*\/\/\s+/, "\n")
        );
      }
      return lines.join(" ");
    })();

    const paramRegex = /@param\s+([A-Za-z0-9_\.]+)\s+(.*)/;
    let params = (() => {
      let params = [];
      let current_param;
      for (let line of this.scratch) {
        let match = line.match(paramRegex);
        if (match) {
          if (current_param) {
            current_param.documentation = current_param.documentation.join(" ");
            params.push(current_param);
          }

          current_param = { label: match[1], documentation: [match[2]] };
        } else {
          if (!/@(?:return|error)/.test(line)) {
            let match = line.match(/\s*(?:\*|\/\/)\s*(.*)/);
            if (match) {
              if (current_param) {
                current_param.documentation.push(match[1]);
              }
            }
          } else {
            if (current_param) {
              current_param.documentation = current_param.documentation.join(
                " "
              );
              params.push(current_param);

              current_param = undefined;
            }
          }
        }
      }

      return params;
    })();

    return { description, params };
  }

  AddVariableCompletion(
    name: string,
    line: string,
    type: string,
    shouldAddToEnumStruct = false,
    funcName: string = undefined
  ): void {
    let range = this.makeDefinitionRange(name, line);
    let scope: string = "$GLOBAL";
    let enumStructName: string = undefined;
    if (this.state.includes(State.EnumStruct)) {
      enumStructName = this.state_data.name;
    }
    if (this.lastFuncLine !== 0) {
      scope = this.lastFuncName;
    }
    if (typeof funcName !== "undefined") {
      scope = funcName;
    }
    // Custom key name for the map so the definitions don't override each others
    let mapName = name + scope + enumStructName;
    if (this.state.includes(State.EnumStruct)) {
      if (this.state.includes(State.Function)) {
        this.completions.add(
          mapName + this.lastFuncName,
          new VariableItem(name, this.file, scope, range, type, enumStructName)
        );
      } else {
        this.completions.add(
          mapName,
          new PropertyItem(
            this.state_data.name,
            name,
            this.file,
            "",
            range,
            type
          )
        );
      }

      return;
    }
    this.completions.add(
      mapName,
      new VariableItem(name, this.file, scope, range, type, "$GLOBAL")
    );
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
    let re = /\s*(?:(?:const|static)\s+)?(\w+)\s*(?:\[(?:[A-Za-z_0-9+* ]*)\])?\s+(\w+)(?:\[(?:[A-Za-z_0-9+* ]*)\])?(?:\s*=\s*(?:[^,]+))?/g;
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
          true,
          funcName
        );
      }
    }
  }

  searchForDefinesInString(line: string): void {
    if (typeof line === "undefined") {
      return;
    }
    let commentIndex = line.length;
    let commentMatch = line.match(/\/\//);
    if (commentMatch) {
      commentIndex = commentMatch.index;
    }
    let matchDefine: RegExpExecArray;
    const re: RegExp = /\w+/g;
    let defineFile: string;
    while ((matchDefine = re.exec(line))) {
      if (matchDefine.index > commentIndex) {
        // We are in a line comment, break.
        break;
      }
      defineFile =
        this.definesMap.get(matchDefine[0]) ||
        this.enumMemberMap.get(matchDefine[0]);
      if (typeof defineFile !== "undefined") {
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
          if (typeof define === "undefined") {
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
        if (typeof items === "undefined") {
          continue;
        }
        let define = items.get(matchDefine[0]);
        if (typeof define === "undefined") {
          continue;
        }
        define.calls.push(location);
        items.add(matchDefine[0], define);
      }
    }
    return;
  }

  getAllMembers(
    itemsRepository: ItemsRepository,
    kind: CompletionItemKind
  ): Map<string, string> {
    let items = itemsRepository.getAllItems(URI.file(this.file).toString());
    if (typeof items === "undefined") {
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
}

function purgeCalls(item: SPItem, file: string): void {
  let uri = URI.file(file);
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

function getParamsFromDeclaration(decl: string): FunctionParam[] {
  let match = decl.match(/\((.+)\)/);
  if (!match) {
    return [];
  }
  // Remove the leading and trailing parenthesis
  decl = match[1] + ",";
  let params: FunctionParam[] = [];
  let re = /\s*((?:const|static)\s+)*\s*(\w+)(?:\[([A-Za-z0-9_\*\+\s\-]*)\])?\:?\s+([A-Za-z0-9_\&]+)\s*(?:\[([A-Za-z0-9_\*\+\s\-]*)\])?\s*(?:\)|,|=)/g;
  let matchVariable;
  while ((matchVariable = re.exec(decl)) != null) {
    params.push({ label: matchVariable[4], documentation: "" });
  }
  return params;
}
