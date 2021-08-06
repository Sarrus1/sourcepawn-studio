import { FileItems } from "./spItemsRepository";
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
  EnumStructMemberItem,
} from "./spItems";
import { isControlStatement } from "./spDefinitions";
import { Range } from "vscode";
import { existsSync, readFileSync } from "fs";
import { basename } from "path";

export function parseFile(
  file: string,
  completions: FileItems,
  documents: Map<string, string>,
  IsBuiltIn: boolean = false
) {
  if (!existsSync(file)) return;
  let data = readFileSync(file, "utf-8");
  parseText(data, file, completions, documents, IsBuiltIn);
}

export function parseText(
  data: string,
  file: string,
  completions: FileItems,
  documents: Map<string, string>,
  IsBuiltIn: boolean = false
) {
  if (typeof data === "undefined") {
    return; // Asked to parse empty file
  }
  let lines = data.split("\n");
  let parser = new Parser(lines, file, IsBuiltIn, completions, documents);
  parser.parse();
}

enum State {
  None,
  MultilineComment,
  DocComment,
  Enum,
  Methodmap,
  Property,
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

  constructor(
    lines: string[],
    file: string,
    IsBuiltIn: boolean,
    completions: FileItems,
    documents: Map<string, string>
  ) {
    this.completions = completions;
    this.state = [State.None];
    this.lineNb = -1;
    this.lines = lines;
    this.file = file;
    this.IsBuiltIn = IsBuiltIn;
    this.documents = documents;
    this.lastFuncLine = 0;
    this.lastFuncName = "";
  }

  parse() {
    let line: string;
    line = this.lines[0];
    while (typeof line != "undefined") {
      this.interpLine(line);
      line = this.lines.shift();
      this.lineNb++;
    }
  }

  interpLine(line: string) {
    if (typeof line === "undefined") return;
    // Match define
    let match = line.match(/\s*#define\s+([A-Za-z0-9_]+)\s+([^]+)/);
    if (match) {
      this.read_define(match, line);
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
      /^\s*(?:(?:new|static|const|decl|public|stock)\s+)*\w+\s+(\w+\s*(?:\[[A-Za-z0-9 +\-\*_]*\])*\s*(?:=\s*[^;,]+)?(?:,|;))/
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
      this.state.push(State.MultilineComment);
      this.scratch = [];

      this.consume_multiline_comment(line, false);
      return;
    }

    match = line.match(/^\s*\/\//);
    if (match) {
      this.state.push(State.MultilineComment);
      this.scratch = [];
      this.consume_multiline_comment(line, true);
      return;
    }

    match = line.match(
      /^\s*methodmap\s+([a-zA-Z][a-zA-Z0-9_]*)(?:\s+<\s+([a-zA-Z][a-zA-Z0-9_]*))?/
    );
    if (match) {
      this.state.push(State.Methodmap);
      this.state_data = {
        name: match[1],
      };
      return;
    }

    // Match properties
    match = line.match(
      /^\s*property\s+([a-zA-Z][a-zA-Z0-9_]*)\s+([a-zA-Z][a-zA-Z0-9_]*)/
    );
    if (match) {
      if (this.state[this.state.length - 1] === State.Methodmap) {
        this.state.push(State.Property);
      }
      this.read_property(match, line);
      return;
    }

    match = line.match(/}/);
    if (match) {
      this.state.pop();
      return;
    }

    // Match functions without description
    match = line.match(
      /(?:(?:static|native|stock|public|forward)\s+)*(?:[a-zA-Z\-_0-9]:)?([^\s]+)\s*([A-Za-z_]*)\s*\(([^\)]*(?:\)?))(?:\s*)(?:\{?)(?:\s*)(?:[^\;\s]*);?\s*$/
    );
    if (match) {
      if (isControlStatement(line)) {
        return;
      }
      let isOldStyle: boolean = match[2] == "";
      this.read_function(line, isOldStyle);
    }
    return;
  }

  read_define(match, line: string) {
    let range = this.makeDefinitionRange(match[1], line);
    this.completions.add(
      match[1],
      new DefineItem(match[1], match[2], this.file, range)
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
      let range = this.makeDefinitionRange(match[1], line);
      var enumStructCompletion: EnumStructItem = new EnumStructItem(
        match[1],
        this.file,
        description,
        range
      );
      this.completions.add(match[1], enumStructCompletion);

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
        match = line.match(/^\s*(?:\w+)\s+(\w+)\s*.*/);

        // Skip if didn't match
        if (!match) {
          continue;
        }
        let enumStructMemberName = match[1];
        // Try to match multiblock comments
        let enumStructMemberDescription: string;
        match = line.match(/\/\*\*<?\s*(.+?(?=\*\/))/);
        if (match) {
          enumStructMemberDescription = match[1];
        }
        match = line.match(/\/\/<?\s*(.*)/);
        if (match) {
          enumStructMemberDescription = match[1];
        }
        let range = this.makeDefinitionRange(enumStructMemberName, line);
        this.completions.add(
          enumStructMemberName + "___property",
          new EnumStructMemberItem(
            enumStructMemberName,
            this.file,
            enumStructMemberDescription,
            enumStructCompletion,
            range
          )
        );
      }
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
            range
          )
        );
      }
      return;
    }
  }

  read_loop_variables(match, line: string) {
    if (this.IsBuiltIn) return;
    this.AddVariableCompletion(match[1], line);
    return;
  }

  read_variables(match, line: string) {
    let match_variables = [];
    let match_variable: RegExpExecArray;
    // Check if it's a multiline declaration
    if (/(;)(?:\s*|)$/.test(line)) {
      // Separate potential multiple declarations
      let re = /\s*(?:(?:const|static|public|stock)\s+)*\w*\s*(?:\[(?:[A-Za-z_0-9+* ]*)\])*\s+(\w+)(?:\[(?:[A-Za-z_0-9+* ]*)\])*(?:\s*=\s*(?:(?:\"[^]*\")|(?:\'[^]*\')|(?:[^,]+)))?/g;
      while ((match_variable = re.exec(line)) != null) {
        match_variables.push(match_variable);
      }
      for (let variable of match_variables) {
        let variable_completion = variable[1].match(
          /(?:\s*)?([A-Za-z_,0-9]*)(?:(?:\s*)?(?:=(?:.*)))?/
        )[1];
        if (!this.IsBuiltIn) {
          this.AddVariableCompletion(variable_completion, line);
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
            this.AddVariableCompletion(variable_completion, line);
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
    let match;
    let iter = 0;
    while (
      typeof current_line != "undefined" &&
      iter < 100 &&
      ((/^\s*\/\//.test(current_line) && use_line_comment) ||
        (!/\*\//.test(current_line) && !use_line_comment))
    ) {
      iter++;
      if (use_line_comment) {
        match = current_line.match(
          /^\s*\/\/\s*@*(?:param|return)*\s*([A-Za-z_\.][A-Za-z0-9_\.]*)\s*(.*)/
        );
      } else {
        match = current_line.match(
          /^\s*\*\s*@*(?:param|return)*\s*([A-Za-z_\.][A-Za-z0-9_\.]*)\s*(.*)/
        );
      }
      this.scratch.push(current_line);
      current_line = this.lines.shift();
      this.lineNb++;
    }
    // Removes the */ from the doc comment
    if (!use_line_comment) {
      current_line = this.lines.shift();
      this.lineNb++;
    }
    this.interpLine(current_line);
    this.state.pop();
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
      line
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

  read_function(line: string, isOldStyle: boolean) {
    if (typeof line === "undefined") {
      return;
    }
    let match: RegExpMatchArray = line.match(
      /^\s*(?:(?:stock|public)\s+)*(?:(\w*)\s+)?(\w*)\s*\((.*(?:\)|,|{))\s*$/
    );
    if (!match) {
      match = line.match(
        /^\s*(?:(?:forward|static|native)\s+)+(\w*\s*:\s*|\w*\s+)?(\w*)\s*\(([^]*)(?:,|;)?\s*$/
      );
    }
    if (match) {
      let { description, params } = this.parse_doc_comment();
      let name_match = match[2];
      if (this.state[this.state.length - 2] === State.Methodmap) {
        this.completions.add(
          name_match + "__method",
          new MethodItem(
            this.state_data.name,
            name_match,
            match[3],
            description,
            params
          )
        );
      } else {
        this.lastFuncLine = this.lineNb;
        this.lastFuncName = name_match;
        let paramsMatch = match[3];
        this.AddParamsDef(paramsMatch, name_match, line);
        // Iteration safety in case something goes wrong
        let maxiter = 0;
        while (
          !paramsMatch.match(/(\))(?:\s*)(?:;)?(?:\s*)(?:\{?)(?:\s*)$/) &&
          typeof line != "undefined" &&
          maxiter < 20
        ) {
          maxiter++;
          line = this.lines.shift();
          this.lineNb++;
          this.AddParamsDef(line, name_match, line);
          paramsMatch += line;
        }
        // Treat differently if the function is declared on multiple lines
        paramsMatch = /\)\s*(?:\{|;)?\s*$/.test(match[0])
          ? match[0]
          : match[0].replace(/\(.*\s*$/, "(") +
            paramsMatch.replace(/\s*\w+\s*\(\s*/g, "").replace(/\s+/gm, " ");
        let range = this.makeDefinitionRange(name_match, line);
        this.completions.add(
          name_match,
          new FunctionItem(
            name_match,
            paramsMatch.replace(/;\s*$/g, ""),
            description,
            params,
            this.file,
            this.IsBuiltIn,
            range
          )
        );
      }
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

  AddVariableCompletion(name: string, line: string): void {
    let range = this.makeDefinitionRange(name, line);
    let scope: string = "$GLOBAL";
    if (this.lastFuncLine !== 0) {
      scope = this.lastFuncName;
    }
    // Custom key name for the map so the definitions don't override each others
    let mapName = name + scope;
    this.completions.add(
      mapName,
      new VariableItem(name, this.file, scope, range)
    );
  }

  makeDefinitionRange(
    name: string,
    line: string,
    search: boolean = true
  ): Range {
    let start: number = search ? line.search(name) : 0;
    let end: number = search ? start + name.length : 0;
    var range = PositiveRange(this.lineNb, start, end);
    return range;
  }

  AddParamsDef(params: string, funcName: string, line: string) {
    let match_variable: RegExpExecArray;
    let match_variables: RegExpExecArray[] = [];
    let re = /\s*(?:(?:const|static)\s+)?\w+\s*(?:\[(?:[A-Za-z_0-9+* ]*)\])?\s+(\w+)(?:\[(?:[A-Za-z_0-9+* ]*)\])?(?:\s*=\s*(?:[^,]+))?/g;
    while ((match_variable = re.exec(params)) != null) {
      match_variables.push(match_variable);
    }
    for (let variable of match_variables) {
      let variable_completion = variable[1].match(
        /(?:\s*)?([A-Za-z_,0-9]*)(?:(?:\s*)?(?:=(?:.*)))?/
      )[1];
      if (!this.IsBuiltIn) {
        this.AddVariableCompletion(variable_completion, line);
      }
    }
  }
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
