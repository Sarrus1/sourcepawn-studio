"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.parse_text = exports.parse_file = void 0;
const smDefinitions = require("./spDefinitions");
const spCompletionsKinds_1 = require("./spCompletionsKinds");
const vscode = require("vscode");
const vscode_uri_1 = require("vscode-uri");
const fs = require("fs");
function parse_file(file, completions, definitions, documents, IsBuiltIn = false) {
    fs.readFile(file, "utf-8", (err, data) => {
        parse_text(data, file, completions, definitions, documents, IsBuiltIn);
    });
}
exports.parse_file = parse_file;
function parse_text(data, file, completions, definitions, documents, IsBuiltIn = false) {
    if (typeof data === "undefined") {
        return; // Asked to parse empty file
    }
    let lines = data.split("\n");
    let parser = new Parser(lines, file, IsBuiltIn, completions, definitions, documents);
    parser.parse();
}
exports.parse_text = parse_text;
var State;
(function (State) {
    State[State["None"] = 0] = "None";
    State[State["MultilineComment"] = 1] = "MultilineComment";
    State[State["DocComment"] = 2] = "DocComment";
    State[State["Enum"] = 3] = "Enum";
    State[State["Methodmap"] = 4] = "Methodmap";
    State[State["Property"] = 5] = "Property";
})(State || (State = {}));
class Parser {
    constructor(lines, file, IsBuiltIn, completions, definitions, documents) {
        this.completions = completions;
        this.definitions = definitions;
        this.state = [State.None];
        this.lineNb = -1;
        this.lines = lines;
        this.file = file;
        this.IsBuiltIn = IsBuiltIn;
        this.documents = documents;
    }
    parse() {
        let line;
        line = this.lines[0];
        while (typeof line != "undefined") {
            this.interpLine(line);
            line = this.lines.shift();
            this.lineNb++;
        }
    }
    interpLine(line) {
        // Match define
        let match = line.match(/\s*#define\s+([A-Za-z0-9_]+)\s+([^]+)/);
        if (match) {
            this.read_define(match);
        }
        // Match global include
        match = line.match(/^\s*#include\s+<([A-Za-z0-9\-_\/.]+)>\s*$/);
        if (match) {
            this.read_include(match, false);
        }
        // Match relative include
        match = line.match(/^\s*#include\s+"([A-Za-z0-9\-_\/.]+)"\s*$/);
        if (match) {
            this.read_include(match, true);
        }
        // Match enums
        match = line.match(/^\s*(?:enum\s+)([A-z0-9_]*)/);
        if (match) {
            this.read_enums(match);
        }
        // Match for loop iteration variable only in the current file
        match = line.match(/^\s*(?:for\s*\(\s*int\s+)([A-z0-9_]*)/);
        if (match && !this.IsBuiltIn) {
            this.read_loop_variables(match);
        }
        // Match variables only in the current file
        match = line.match(/^(?:\s*)?(?:bool|char|const|float|int|any|Plugin|Handle|ConVar|Cookie|Database|DBDriver|DBResultSet|DBStatement|GameData|Transaction|Event|File|DirectoryListing|KeyValues|Menu|Panel|Protobuf|Regex|SMCParser|TopMenu|Timer|FrameIterator|GlobalForward|PrivateForward|Profiler)\s+(.*)/);
        if (match && !this.IsBuiltIn) {
            this.read_variables(match);
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
        match = line.match(/^\s*methodmap\s+([a-zA-Z][a-zA-Z0-9_]*)(?:\s+<\s+([a-zA-Z][a-zA-Z0-9_]*))?/);
        if (match) {
            this.state.push(State.Methodmap);
            this.state_data = {
                name: match[1],
            };
            return;
        }
        // Match properties
        match = line.match(/^\s*property\s+([a-zA-Z][a-zA-Z0-9_]*)\s+([a-zA-Z][a-zA-Z0-9_]*)/);
        if (match) {
            if (this.state[this.state.length - 1] === State.Methodmap) {
                this.state.push(State.Property);
            }
            return;
        }
        match = line.match(/}/);
        if (match) {
            this.state.pop();
            return;
        }
        // Match functions without description
        match = line.match(/(?:(?:static|native|stock|public|forward)+\s*)+\s+(?:[a-zA-Z\-_0-9]:)?([^\s]+)\s*([A-Za-z_]*)\(([^\)]*)(?:\)?)(?:\s*)(?:\{?)(?:\s*)(?:[^\;\s]*)$/);
        if (match && !this.IsBuiltIn) {
            this.read_non_descripted_function(match, "");
        }
        return;
    }
    read_define(match) {
        this.completions.add(match[1], new spCompletionsKinds_1.DefineCompletion(match[1], match[2], this.file));
        let def = new smDefinitions.DefLocation(vscode_uri_1.URI.file(this.file), new vscode.Range(this.lineNb, 0, this.lineNb, 0), smDefinitions.DefinitionKind.Define);
        this.definitions.set(match[1], def);
        return;
    }
    read_include(match, isRelative) {
        this.completions.resolve_import(match[1], this.documents, this.IsBuiltIn);
        return;
    }
    read_enums(match) {
        // Create a completion for the enum itself
        let enumCompletion = new spCompletionsKinds_1.EnumCompletion(match[1], this.file);
        this.completions.add(match[1], enumCompletion);
        let def = new smDefinitions.DefLocation(vscode_uri_1.URI.file(this.file), new vscode.Range(this.lineNb, 0, this.lineNb, 0), smDefinitions.DefinitionKind.Enum);
        this.definitions.set(match[1], def);
        // Set max number of iterations for safety
        let iter = 0;
        // Proceed to the next line
        let line = "";
        // Match all the params of the enum
        while (iter < 20 && !line.match(/^\s*(\}\s*\;)/)) {
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
            this.completions.add(match[1], new spCompletionsKinds_1.EnumMemberCompletion(match[1], this.file, enumCompletion));
            let def = new smDefinitions.DefLocation(vscode_uri_1.URI.file(this.file), new vscode.Range(this.lineNb, 0, this.lineNb, 0), smDefinitions.DefinitionKind.EnumMember);
            this.definitions.set(match[1], def);
        }
        return;
    }
    read_loop_variables(match) {
        this.completions.add(match[1], new spCompletionsKinds_1.VariableCompletion(match[1], this.file));
        return;
    }
    read_variables(match) {
        let match_variables = [];
        // Check if it's a multiline declaration
        if (match[1].match(/(;)(?:\s*|)$/)) {
            // Separate potential multiple declarations
            match_variables = match[1].match(/(?:\s*)?([A-z0-9_\[`\]]+(?:\s+)?(?:\=(?:(?:\s+)?(?:[\(].*?[\)]|[\{].*?[\}]|[\"].*?[\"]|[\'].*?[\'])?(?:[A-z0-9_\[`\]]*)))?(?:\s+)?|(!,))/g);
            for (let variable of match_variables) {
                let variable_completion = variable.match(/(?:\s*)?([A-Za-z_,0-9]*)(?:(?:\s*)?(?:=(?:.*)))?/)[1];
                this.completions.add(variable_completion, new spCompletionsKinds_1.VariableCompletion(variable_completion, this.file));
                // Save as definition if it's a global variable
                if (/g_.*/g.test(variable_completion)) {
                    let def = new smDefinitions.DefLocation(vscode_uri_1.URI.file(this.file), new vscode.Range(this.lineNb, 0, this.lineNb, 0), smDefinitions.DefinitionKind.Variable);
                    this.definitions.set(variable_completion, def);
                }
            }
        }
        else {
            while (!match[1].match(/(;)(?:\s*|)$/)) {
                // Separate potential multiple declarations
                match_variables = match[1].match(/(?:\s*)?([A-z0-9_\[`\]]+(?:\s+)?(?:\=(?:(?:\s+)?(?:[\(].*?[\)]|[\{].*?[\}]|[\"].*?[\"]|[\'].*?[\'])?(?:[A-z0-9_\[`\]]*)))?(?:\s+)?|(!,))/g);
                if (!match_variables) {
                    break;
                }
                for (let variable of match_variables) {
                    let variable_completion = variable.match(/(?:\s*)?([A-Za-z_,0-9]*)(?:(?:\s*)?(?:=(?:.*)))?/)[1];
                    this.completions.add(variable_completion, new spCompletionsKinds_1.VariableCompletion(variable_completion, this.file));
                    // Save as definition if it's a global variable
                    if (/g_.*/g.test(variable_completion)) {
                        let def = new smDefinitions.DefLocation(vscode_uri_1.URI.file(this.file), new vscode.Range(this.lineNb, 0, this.lineNb, 0), smDefinitions.DefinitionKind.Variable);
                        this.definitions.set(variable_completion, def);
                    }
                }
                match[1] = this.lines.shift();
                this.lineNb++;
            }
        }
        return;
    }
    read_non_descripted_function(match, description = "") {
        let match_buffer = "";
        let line = "";
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
        // Save as definition
        let def = new smDefinitions.DefLocation(vscode_uri_1.URI.file(this.file), new vscode.Range(this.lineNb, 0, this.lineNb, 0), smDefinitions.DefinitionKind.Function);
        this.definitions.set(name_match, def);
        match_buffer = match[0];
        // Check if function takes arguments
        let maxiter = 0;
        while (!match_buffer.match(/(\))(?:\s*)(?:;)?(?:\s*)(?:\{?)(?:\s*)$/) &&
            maxiter < 20) {
            line = this.lines.shift();
            this.lineNb++;
            if (typeof line === "undefined") {
                return;
            }
            //partial_params_match += line;
            match_buffer += line;
            maxiter++;
        }
        let params = [];
        let current_param;
        if (params_match) {
            for (let param of params_match) {
                current_param = {
                    label: param,
                    documentation: param,
                };
                params.push(current_param);
                // Add the params as variables
                let paramAsVariable = param.match(/([^\s:]*)$/)[1];
                this.completions.add(paramAsVariable, new spCompletionsKinds_1.VariableCompletion(paramAsVariable, this.file));
            }
        }
        this.completions.add(name_match, new spCompletionsKinds_1.FunctionCompletion(name_match, match_buffer.replace(/;\s*$/g, ""), description, params, this.file, this.IsBuiltIn));
        return;
    }
    consume_multiline_comment(current_line, use_line_comment = false) {
        if (typeof current_line === "undefined") {
            return; // EOF
        }
        let match = use_line_comment
            ? !/^\s*\/\//.test(current_line)
            : /\*\//.test(current_line);
        if (match) {
            if (this.state[this.state.length - 1] === State.DocComment) {
                this.state.pop();
                this.state.pop();
                if (use_line_comment) {
                    //return this.read_function(current_line);
                    if (/\s*(?:(?:static|native|stock|public|forward)+\s*)+\s+([^\s]+)\s*([A-Za-z_].*)/.test(current_line) || /\s*(?:(?:static|native|stock|public|forward)+\s*)+\s+(?:[a-zA-Z\-_0-9]:)?([^\s]+)\s*\(\s*([A-Za-z_].*)/.test(current_line)) {
                        return this.read_function(current_line);
                    }
                    else {
                        this.interpLine(current_line);
                        return;
                    }
                }
                else {
                    current_line = this.lines.shift();
                    this.lineNb++;
                    if (!(typeof current_line === "undefined")) {
                        if (/\s*(?:(?:static|native|stock|public|forward)+\s*)+\s+([^\s]+)\s*([A-Za-z_].*)/.test(current_line) || /\s*(?:(?:static|native|stock|public|forward)+\s*)+\s+(?:[a-zA-Z\-_0-9]:)?([^\s]+)\s*\(\s*([A-Za-z_].*)/.test(current_line)) {
                            return this.read_function(current_line);
                        }
                        else {
                            this.interpLine(current_line);
                            return;
                        }
                    }
                }
            }
            this.state.pop();
            return;
        }
        else {
            if (!use_line_comment) {
                match = current_line.match(/^\s*\*\s*@*(?:param|return)*\s*([A-Za-z_\.][A-Za-z0-9_\.]*)\s*(.*)/);
            }
            else {
                match = current_line.match(/^\s*\/\/\s*@*(?:param|return)*\s*([A-Za-z_\.][A-Za-z0-9_\.]*)\s*(.*)/);
            }
            if (match) {
                if (this.state[this.state.length - 1] !== State.DocComment) {
                    this.state.push(State.DocComment);
                }
            }
            this.scratch.push(current_line);
            current_line = this.lines.shift();
            this.lineNb++;
            if (!(typeof current_line === "undefined")) {
                this.consume_multiline_comment(current_line, use_line_comment);
            }
        }
    }
    clean_param(partial_params_match) {
        let unused_comma = partial_params_match.match(/(\))(?:\s*)(?:;)?(?:\s*)$/);
        if (unused_comma) {
            partial_params_match = partial_params_match.replace(unused_comma[1], "");
        }
        return partial_params_match;
    }
    read_function(line) {
        if (typeof line === "undefined") {
            return;
        }
        if (line.includes(":")) {
            this.read_old_style_function(line);
        }
        else {
            this.read_new_style_function(line);
        }
        this.state.pop();
        return;
    }
    read_old_style_function(line) {
        let match = line.match(/\s*(?:(?:static|native|stock|public|forward)+\s*)+\s+(?:[a-zA-Z\-_0-9]:)?([^\s]+)\s*\(\s*([A-Za-z_].*)/);
        if (match) {
            let def = new smDefinitions.DefLocation(vscode_uri_1.URI.file(this.file), new vscode.Range(this.lineNb, 0, this.lineNb, 0), smDefinitions.DefinitionKind.Function);
            this.definitions.set(match[1], def);
            let { description, params } = this.parse_doc_comment();
            this.completions.add(match[1], new spCompletionsKinds_1.FunctionCompletion(match[1], match[0].replace(/;\s*$/g, ""), description, params, this.file, this.IsBuiltIn));
        }
    }
    read_new_style_function(line) {
        let match = line.match(/\s*(?:(?:static|native|stock|public|forward)+\s*)+\s+([^\s]+)\s*([A-Za-z_].*)/);
        if (match) {
            let { description, params } = this.parse_doc_comment();
            let name_match = match[2].match(/^([A-Za-z_][A-Za-z0-9_]*)/);
            let def = new smDefinitions.DefLocation(vscode_uri_1.URI.file(this.file), new vscode.Range(this.lineNb, 0, this.lineNb, 0), smDefinitions.DefinitionKind.Function);
            this.definitions.set(name_match[1], def);
            if (this.state[this.state.length - 1] === State.Methodmap) {
                this.completions.add(name_match[1], new spCompletionsKinds_1.MethodCompletion(this.state_data.name, name_match[1], match[2], description, params));
            }
            else {
                let paramsMatch = match[2];
                // Iteration safety in case something goes wrong
                let maxiter = 0;
                let line;
                line = this.lines.shift();
                this.lineNb++;
                while (!paramsMatch.match(/(\))(?:\s*)(?:;)?(?:\s*)(?:\{?)(?:\s*)$/) &&
                    (typeof line != "undefined") &&
                    maxiter < 20) {
                    paramsMatch += line;
                    maxiter++;
                    line = this.lines.shift();
                    this.lineNb++;
                }
                // Treat differently if the function is declared on multiple lines
                paramsMatch = /\)\s*(?:\{|;)?\s*$/.test(match[0]) ? match[0] : match[0] + paramsMatch.replace(/\s*[A-z0-9_]+\s*\(\s*/g, "").replace(/\s+/gm, " ");
                this.completions.add(name_match[1], new spCompletionsKinds_1.FunctionCompletion(name_match[1], paramsMatch.replace(/;\s*$/g, ""), description, params, this.file, this.IsBuiltIn));
            }
        }
    }
    parse_doc_comment() {
        let description = (() => {
            let lines = [];
            for (let line of this.scratch) {
                //Check if @return or @error
                if (/^\s*\/\*\*\s*/.test(line)) {
                    continue;
                }
                //if (!(/^\s*\*\s*(@(?!param)|[^@])*$/.test(line) || /^\s*\/\/\s*(@(?!param)|[^@])*$/.test(line))) 
                //{
                //continue;
                //}
                lines.push(line.replace(/^\s*\*\s+/, "\n").replace(/^\s*\/\/\s+/, "\n"));
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
                }
                else {
                    if (!/@(?:return|error)/.test(line)) {
                        let match = line.match(/\s*(?:\*|\/\/)\s*(.*)/);
                        if (match) {
                            if (current_param) {
                                current_param.documentation.push(match[1]);
                            }
                        }
                    }
                    else {
                        if (current_param) {
                            current_param.documentation = current_param.documentation.join(" ");
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
//# sourceMappingURL=spParser.js.map