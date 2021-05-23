import * as spCompletions from "./spCompletions";
import * as spDefinitions from "./spDefinitions";
import {
  FunctionCompletion,
  DefineCompletion,
  EnumCompletion,
  EnumMemberCompletion,
  VariableCompletion,
  MethodCompletion,
  FunctionParam,
  PropertyCompletion,
  EnumStructCompletion,
  EnumStructMemberCompletion,
} from "./spCompletionsKinds";
import * as vscode from "vscode";
import { URI } from "vscode-uri";
import * as fs from "fs";
import { basename } from "path";

export function parse_file(
  file: string,
  completions: spCompletions.FileCompletions,
  definitions: spDefinitions.Definitions,
  documents: Map<string, URI>,
  IsBuiltIn: boolean = false
) {
  let data = fs.readFileSync(file, "utf-8");
  parse_text(data, file, completions, definitions, documents, IsBuiltIn);
}

export function parse_text(
  data: string,
  file: string,
  completions: spCompletions.FileCompletions,
  definitions: spDefinitions.Definitions,
  documents: Map<string, URI>,
  IsBuiltIn: boolean = false
) {
  if (typeof data === "undefined") {
    return; // Asked to parse empty file
  }
  let lines = data.split("\n");
  let parser = new Parser(
    lines,
    file,
    IsBuiltIn,
    completions,
    definitions,
    documents
  );
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
  completions: spCompletions.FileCompletions;
  definitions: spDefinitions.Definitions;
  state: State[];
  scratch: any;
  state_data: any;
  lines: string[];
  lineNb: number;
  file: string;
  IsBuiltIn: boolean;
  documents: Map<string, URI>;
  lastFuncLine: number;
  lastFuncName: string;

  constructor(
    lines: string[],
    file: string,
    IsBuiltIn: boolean,
    completions: spCompletions.FileCompletions,
    definitions: spDefinitions.Definitions,
    documents: Map<string, URI>
  ) {
    this.completions = completions;
    this.definitions = definitions;
    let uri = URI.file(file).toString();
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
      this.read_define(match);
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
      this.read_loop_variables(match);
      return;
    }

    // Match variables only in the current file
    match = line.match(
      /^\s*(?:(?:new|static|const|decl|public|stock)\s+)*[A-z0-9_]+\s+(\w+\s*(?:\[[A-Za-z0-9 +\-\*_]*\])*\s*(?:=\s*[^;,]+)?(?:,|;))/
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
      this.read_property(match);
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
      let testWords = ["if", "else", "for", "while", "function"];
      for (let word of testWords) {
        let regExp = new RegExp(`\\b${word}\\b`);
        if (regExp.test(match[1]) || regExp.test(match[2])) return;
      }

      let isOldStyle: boolean = match[2] == "";
      this.read_function(line, isOldStyle);
    }
    return;
  }

  read_define(match) {
    this.completions.add(
      match[1],
      new DefineCompletion(match[1], match[2], this.file)
    );
    let def: spDefinitions.DefLocation = new spDefinitions.DefLocation(
      URI.file(this.file),
      PositiveRange(this.lineNb),
      spDefinitions.DefinitionKind.Define
    );
    this.AddDefinition(match[1], def);
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
      var enumStructCompletion: EnumStructCompletion = new EnumStructCompletion(
        match[1],
        this.file,
        description
      );
      this.completions.add(match[1], enumStructCompletion);
      let start: number = 0;
      let end: number = 0;
      if (match[1] == "") {
        start = line.search(match[1]);
        end = start + match[1].length;
      }
      var def: spDefinitions.DefLocation = new spDefinitions.DefLocation(
        URI.file(this.file),
        PositiveRange(this.lineNb, start, end),
        spDefinitions.DefinitionKind.EnumStruct
      );
      this.AddDefinition(match[1], def);

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
        match = line.match(/^\s*(?:[A-z0-9_]*)\s+([A-z0-9_]*)\s*.*/);

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
        this.completions.add(
          enumStructMemberName + "___property",
          new EnumStructMemberCompletion(
            enumStructMemberName,
            this.file,
            enumStructMemberDescription,
            enumStructCompletion
          )
        );
        let start: number = line.search(enumStructMemberName);
        let end: number = start + enumStructMemberName.length;
        let def: spDefinitions.DefLocation = new spDefinitions.DefLocation(
          URI.file(this.file),
          PositiveRange(this.lineNb, start, end),
          spDefinitions.DefinitionKind.EnumStructMember
        );
        this.AddDefinition(enumStructMemberName, def);
      }
    } else {
      let nameMatch = match[0].match(/^\s*(?:enum\s*)([A-z0-9_]*)/);
      if (nameMatch) {
        // Create a completion for the enum itself if it has a name
        var enumCompletion: EnumCompletion = new EnumCompletion(
          nameMatch[1],
          this.file,
          description
        );
        this.completions.add(nameMatch[1], enumCompletion);
        let start: number = 0;
        let end: number = 0;
        if (match[1] == "") {
          start = line.search(match[1]);
          end = start + match[1].length;
        }
        var def: spDefinitions.DefLocation = new spDefinitions.DefLocation(
          URI.file(this.file),
          PositiveRange(this.lineNb, start, end),
          spDefinitions.DefinitionKind.Enum
        );
        this.AddDefinition(match[1], def);
      } else {
        var enumCompletion: EnumCompletion = new EnumCompletion(
          "",
          this.file,
          description
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
        match = line.match(/^\s*([A-z0-9_]*)\s*.*/);

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
        this.completions.add(
          enumMemberName,
          new EnumMemberCompletion(
            enumMemberName,
            this.file,
            enumMemberDescription,
            enumCompletion
          )
        );
        let start: number = line.search(enumMemberName);
        let end: number = start + enumMemberName.length;
        let def: spDefinitions.DefLocation = new spDefinitions.DefLocation(
          URI.file(this.file),
          PositiveRange(this.lineNb, start, end),
          spDefinitions.DefinitionKind.EnumMember
        );
        this.AddDefinition(enumMemberName, def);
      }
      return;
    }
  }

  read_loop_variables(match) {
    this.completions.add(match[1], new VariableCompletion(match[1], this.file));
    return;
  }

  read_variables(match, line: string) {
    let match_variables = [];
    let match_variable: RegExpExecArray;
    // Check if it's a multiline declaration
    if (/(;)(?:\s*|)$/.test(line)) {
      // Separate potential multiple declarations
      let re = /\s*(?:(?:const|static|public)\s+)*\w+\s*(?:\[(?:[A-Za-z_0-9+* ]*)\])*\s+(\w+)(?:\[(?:[A-Za-z_0-9+* ]*)\])*(?:\s*=\s*(?:(?:\"[^]*\")|(?:\'[^]*\')|(?:[^,]+)))?/g;
      while ((match_variable = re.exec(line)) != null) {
        match_variables.push(match_variable);
      }
      for (let variable of match_variables) {
        let variable_completion = variable[1].match(
          /(?:\s*)?([A-Za-z_,0-9]*)(?:(?:\s*)?(?:=(?:.*)))?/
        )[1];
        this.completions.add(
          variable_completion,
          new VariableCompletion(variable_completion, this.file)
        );
        if (this.lastFuncLine == 0) {
          let start: number = line.search(variable_completion);
          let end: number = start + variable_completion.length;
          var def: spDefinitions.DefLocation = new spDefinitions.DefLocation(
            URI.file(this.file),
            PositiveRange(this.lineNb, start, end),
            spDefinitions.DefinitionKind.Variable
          );
          this.AddDefinition(variable_completion, def);
        } else {
          let start: number = line.search(variable_completion);
          let end: number = start + variable_completion.length;
          var def: spDefinitions.DefLocation = new spDefinitions.DefLocation(
            URI.file(this.file),
            PositiveRange(this.lineNb, start, end),
            spDefinitions.DefinitionKind.Variable,
            this.lastFuncName
          );
          this.AddDefinition(variable_completion, def, this.lastFuncName);
        }
      }
    } else {
      while (!match[1].match(/(;)(?:\s*|)$/)) {
        // Separate potential multiple declarations
        match_variables = match[1].match(
          /(?:\s*)?([A-z0-9_\[`\]]+(?:\s+)?(?:\=(?:(?:\s+)?(?:[\(].*?[\)]|[\{].*?[\}]|[\"].*?[\"]|[\'].*?[\'])?(?:[A-z0-9_\[`\]]*)))?(?:\s+)?|(!,))/g
        );
        if (!match_variables) {
          break;
        }
        for (let variable of match_variables) {
          let variable_completion = variable.match(
            /(?:\s*)?([A-Za-z_,0-9]*)(?:(?:\s*)?(?:=(?:.*)))?/
          )[1];
          this.completions.add(
            variable_completion,
            new VariableCompletion(variable_completion, this.file)
          );
          // Save as definition if it's a global variable
          if (/g_.*/g.test(variable_completion)) {
            let def: spDefinitions.DefLocation = new spDefinitions.DefLocation(
              URI.file(this.file),
              PositiveRange(this.lineNb),
              spDefinitions.DefinitionKind.Variable
            );
            this.AddDefinition(variable_completion, def);
          }
        }
        match[1] = this.lines.shift();
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

  read_property(match) {
    let { description, params } = this.parse_doc_comment();
    let name_match: string = match[2];
    let NewPropertyCompletion = new PropertyCompletion(
      this.state_data.name,
      name_match,
      this.file,
      description
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
      /^\s*(?:(?:stock|public)\s+)*(?:(\w*)\s+)?(\w*)\s*\(([^]*(?:\)|,|{))\s*$/
    );
    if (!match) {
      match = line.match(
        /^\s*(?:(?:forward|static|native)\s+)+(?:(\w*)\s+)?(\w*)\s*\(([^]*)(?:,|;)?\s*$/
      );
    }
    if (match) {
      let { description, params } = this.parse_doc_comment();
      let name_match = match[2];
      let start: number = line.search(name_match);
      let end: number = start + name_match.length;
      let def: spDefinitions.DefLocation = new spDefinitions.DefLocation(
        URI.file(this.file),
        PositiveRange(this.lineNb, start, end),
        spDefinitions.DefinitionKind.Function
      );
      this.AddDefinition(name_match, def);
      if (this.state[this.state.length - 2] === State.Methodmap) {
        this.completions.add(
          name_match + "__method",
          new MethodCompletion(
            this.state_data.name,
            name_match,
            match[3],
            description,
            params
          )
        );
      } else {
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
            paramsMatch
              .replace(/\s*[A-z0-9_]+\s*\(\s*/g, "")
              .replace(/\s+/gm, " ");
        this.lastFuncLine = this.lineNb;
        this.lastFuncName = name_match;
        this.completions.add(
          name_match,
          new FunctionCompletion(
            name_match,
            paramsMatch.replace(/;\s*$/g, ""),
            description,
            params,
            this.file,
            this.IsBuiltIn
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

  AddDefinition(
    name: string,
    def: spDefinitions.DefLocation,
    definitionSuffix: string = "___gLobaL"
  ): void {
    if (definitionSuffix != "___gLobaL")
      definitionSuffix = "___" + definitionSuffix;
    if (!this.definitions.has(name + definitionSuffix) || !this.IsBuiltIn) {
      this.definitions.set(name + definitionSuffix, def);
    }
    return;
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
      this.completions.add(
        variable_completion,
        new VariableCompletion(variable_completion, this.file)
      );
      let start: number = line.search(variable_completion);
      let end: number = start + variable_completion.length;
      var def: spDefinitions.DefLocation = new spDefinitions.DefLocation(
        URI.file(this.file),
        PositiveRange(this.lineNb, start, end),
        spDefinitions.DefinitionKind.Variable,
        funcName
      );
      this.AddDefinition(variable_completion, def, funcName);
    }
  }
}

function PositiveRange(
  lineNb: number,
  start: number = 0,
  end: number = 0
): vscode.Range {
  lineNb = lineNb > 0 ? lineNb : 0;
  return new vscode.Range(lineNb, start, lineNb, end);
}

function IsIncludeSelfFile(file: string, include: string): boolean {
  let baseName: string = basename(file);
  let match = include.match(/([A-z0-9]*)(?:.sp|.inc)?$/);
  if (match) {
    return baseName == match[1];
  }
  return false;
}
