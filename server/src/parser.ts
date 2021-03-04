import {
  FileCompletions,
  FunctionCompletion,
  DefineCompletion,
  FunctionParam,
  MethodCompletion,
  VariableCompletion,
} from "./completions";
import * as fs from "fs";

export function parse_file(file: string, completions: FileCompletions) {
  fs.readFile(file, "utf-8", (err, data) => {
    parse_blob(data, completions, file);
  });
}

export function parse_blob(
  data: string,
  completions: FileCompletions,
  file = ""
) {
  if (typeof data === "undefined") {
    return; // Asked to parse empty file
  }
  let lines = data.split("\n");
  let parser = new Parser(lines, completions);
  parser.parse(file);
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
  lines: string[];
  completions: FileCompletions;
  state: State[];
  scratch: any;
  state_data: any;

  constructor(lines: string[], completions: FileCompletions) {
    this.lines = lines;
    this.completions = completions;
    this.state = [State.None];
  }

  parse(file) {
    let line = this.lines.shift();
    if (typeof line === "undefined") {
      return;
    }

    let match = line.match(/\s*#define\s+([A-Za-z0-9_]+)/);
    if (match) {
      this.completions.add(match[1], new DefineCompletion(match[1]));
      return this.parse(file);
    }

    match = line.match(/^\s*#include\s+<([A-Za-z0-9\-_\/]+)>\s*$/);
    if (match) {
      this.completions.resolve_import(match[1], false);
      return this.parse(file);
    }

    match = line.match(/^\s*#include\s+"([A-Za-z0-9\-_\/]+)"\s*$/);
    if (match) {
      this.completions.resolve_import(match[1], true);
      return this.parse(file);
    }

    // Match variables only in the current file
    match = line.match(
      /^(?:\s*)?(?:bool|char|const|float|int|anyPlugin|Handle|ConVar|Cookie|Database|DBDriver|DBResultSet|DBStatement|GameData|Transaction|Event|File|DirectoryListing|KeyValues|Menu|Panel|Protobuf|Regex|SMCParser|TopMenu|Timer|FrameIterator|GlobalForward|PrivateForward|Profiler)\s+(.*)/
    );
    if (match) {
      let match_variables = [];
      // Check if it's a multiline declaration
      if (match[1].match(/(;)(?:\s*|)$/)) {
        // Separate potential multiple declarations
        match_variables = match[1].match(
          /(?:\s*)?([A-z0-9_\[`\]]+(?:\s+)?(?:\=(?:(?:\s+)?(?:[\(].*?[\)]|[\{].*?[\}]|[\"].*?[\"]|[\'].*?[\'])?(?:[A-z0-9_\[`\]]*)))?(?:\s+)?|(!,))/g
        );
        console.debug("test1", match_variables);
        for (let variable of match_variables) {
          let variable_completion = variable.match(
            /(?:\s*)?([A-Za-z_,0-9]*)(?:(?:\s*)?(?:=(?:.*)))?/
          )[1];
          this.completions.add(
            variable_completion,
            new VariableCompletion(variable_completion, file)
          );
        }
      } else {
        console.debug(line, match);
        while (!match[1].match(/(;)(?:\s*|)$/)) {
          // Separate potential multiple declarations
          match_variables = match[1].match(
            /(?:\s*)?([A-z0-9_\[`\]]+(?:\s+)?(?:\=(?:(?:\s+)?(?:[\(].*?[\)]|[\{].*?[\}]|[\"].*?[\"]|[\'].*?[\'])?(?:[A-z0-9_\[`\]]*)))?(?:\s+)?|(!,))/g
          );
          console.debug("test2", match_variables);
          if (!match_variables) {
            break;
          }
          for (let variable of match_variables) {
            let variable_completion = variable.match(
              /(?:\s*)?([A-Za-z_,0-9]*)(?:(?:\s*)?(?:=(?:.*)))?/
            )[1];
            this.completions.add(
              variable_completion,
              new VariableCompletion(variable_completion, file)
            );
          }
          match[1] = this.lines.shift();
        }
      }

      return this.parse(file);
    }

    match = line.match(/\s*\/\*/);
    if (match) {
      this.state.push(State.MultilineComment);
      this.scratch = [];

      this.consume_multiline_comment(line, false, file);
      return this.parse(file);
    }

    match = line.match(/^\s*\/\//);
    if (match) {
      if (this.lines[0] && this.lines[0].match(/^\s*\/\//)) {
        this.state.push(State.MultilineComment);
        this.scratch = [];

        this.consume_multiline_comment(line, true, file);
        return this.parse(file);
      }
    }

    match = line.match(
      /^\s*methodmap\s+([a-zA-Z][a-zA-Z0-9_]*)(?:\s+<\s+([a-zA-Z][a-zA-Z0-9_]*))?/
    );
    if (match) {
      this.state.push(State.Methodmap);
      this.state_data = {
        name: match[1],
      };

      return this.parse(file);
    }

    // Match properties
    match = line.match(
      /^\s*property\s+([a-zA-Z][a-zA-Z0-9_]*)\s+([a-zA-Z][a-zA-Z0-9_]*)/
    );
    if (match) {
      if (this.state[this.state.length - 1] === State.Methodmap) {
        this.state.push(State.Property);
      }

      return this.parse(file);
    }

    // Match new style functions without description
    match = line.match(
      /(?:(?:static|native|stock|public|\n)+\s*)+\s+(?:[a-zA-Z\-_0-9]:)?([^\s]+)\s*([A-Za-z_]*)\((.*)\)(?:\s|\{|)(?!;)$/
    );
    if (match) {
      let name_match = "";
      let params_match = [];
      // Separation for old and new style functions
      // New style
      if (match[2] != "") {
        name_match = match[2];
      }
      // Old style
      else {
        name_match = match[1];
      }

      // Check if function takes arguments
      if (match[3]) {
        params_match = match[3].match(/([^,\)]+\(.+?\))|([^,\)]+)/g);
      }
      let params = [];
      let current_param;
      for (let param of params_match) {
        current_param = {
          label: param,
          documentation: param,
        };
        params.push(current_param);
        // Add the params as variables
        let paramAsVariable = param.match(/([^\s:]*)$/)[1];
        this.completions.add(
          paramAsVariable,
          new VariableCompletion(paramAsVariable, file)
        );
      }
      this.completions.add(
        name_match,
        new FunctionCompletion(name_match, name_match, "", params)
      );
      return this.parse(file);
    }

    match = line.match(/}/);
    if (match) {
      this.state.pop();

      return this.parse(file);
    }

    this.parse(file);
  }

  consume_multiline_comment(
    current_line: string,
    use_line_comment: boolean = false,
    file: string
  ) {
    if (typeof current_line === "undefined") {
      return; // EOF
    }

    let match: any = use_line_comment
      ? !/^\s*\/\//.test(current_line)
      : /\*\//.test(current_line);
    if (match) {
      if (this.state[this.state.length - 1] === State.DocComment) {
        this.state.pop();
        this.state.pop();

        if (use_line_comment) {
          return this.read_function(current_line, file);
        } else {
          return this.read_function(this.lines.shift(), file);
        }
      }

      this.state.pop();
      return this.parse(file);
    } else {
      if (!use_line_comment) {
        match = current_line.match(
          /^\s*\*\s*@(?:param|return)\s*([A-Za-z_\.][A-Za-z0-9_\.]*)\s*(.*)/
        );
      } else {
        match = current_line.match(
          /^\s*\/\/\s*@(?:param|return)\s*([A-Za-z_\.][A-Za-z0-9_\.]*)\s*(.*)/
        );
      }

      if (match) {
        if (this.state[this.state.length - 1] !== State.DocComment) {
          this.state.push(State.DocComment);
        }
      }

      this.scratch.push(current_line);

      this.consume_multiline_comment(
        this.lines.shift(),
        use_line_comment,
        file
      );
    }
  }

  read_function(line: string, file: string) {
    if (typeof line === "undefined") {
      return;
    }

    // TODO: Support multiline function definitions
    if (line.includes(":")) {
      this.read_old_style_function(line);
    } else {
      this.read_new_style_function(line);
    }

    this.state.pop();
    this.parse(file);
  }

  read_old_style_function(line: string) {
    let match = line.match(
      /\s*(?:(?:static|native|stock|public)+\s*)+\s+(?:[a-zA-Z\-_0-9]:)?([^\s]+)\s*\(\s*([A-Za-z_].*)/
    );
    if (match) {
      let { description, params } = this.parse_doc_comment();
      this.completions.add(
        match[1],
        new FunctionCompletion(match[1], match[2], description, params)
      );
    }
  }

  read_new_style_function(line: string) {
    let match = line.match(
      /\s*(?:(?:static|native|stock|public)+\s*)+\s+([^\s]+)\s*([A-Za-z_].*)/
    );
    if (match) {
      let { description, params } = this.parse_doc_comment();
      let name_match = match[2].match(/^([A-Za-z_][A-Za-z0-9_]*)/);
      if (this.state[this.state.length - 1] === State.Methodmap) {
        this.completions.add(
          name_match[1],
          new MethodCompletion(
            this.state_data.name,
            name_match[1],
            match[2],
            description,
            params
          )
        );
      } else {
        this.completions.add(
          name_match[1],
          new FunctionCompletion(name_match[1], match[2], description, params)
        );
      }
    }
  }

  parse_doc_comment(): { description: string; params: FunctionParam[] } {
    let description = (() => {
      let lines = [];
      for (let line of this.scratch) {
        if (
          !(/^\s*\*\s+([^@].*)/.test(line) || /^\s*\/\/\s+([^@].*)/.test(line))
        ) {
          break;
        }

        lines.push(line.replace(/^\s*\*\s+/, "").replace(/^\s*\/\/\s+/, ""));
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
}
