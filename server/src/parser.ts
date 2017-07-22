import { FileCompletions, FunctionCompletion, DefineCompletion } from './completions';
import * as fs from 'fs';

export function parse_file(file: string, completions: FileCompletions) {
    fs.readFile(file, "utf-8", (err, data) => {
        parse_blob(data, completions);
    });
}

export function parse_blob(data: string, completions: FileCompletions) {
    let lines = data.split("\n");
    let parser = new Parser(lines, completions);

    parser.parse();
}

enum State {
    None,
    MultilineComment,
    DocComment,
    Enum,
}

class Parser {
    lines: string[];
    completions: FileCompletions;
    state: State;
    scratch: any;

    constructor(lines: string[], completions: FileCompletions) {
        this.lines = lines;
        this.completions = completions;
        this.state = State.None;
    }

    parse() {
        let line = this.lines.shift();
        if (typeof line === 'undefined') {
            return;
        }

        let match = line.match(/\s*#define\s+([A-Za-z0-9_]+)/);
        if (match) {
            this.completions.add(match[1], new DefineCompletion(match[1]));
            this.parse();
        }

        match = line.match(/^\s*#include\s+<([A-Za-z0-9\-_\/]+)>\s*$/);
        if (match) {
            this.completions.resolve_import(match[1]);
            this.parse();
        }

        match = line.match(/^\s*#include\s+"([A-Za-z0-9\-_\/]+)"\s*$/);
        if (match) {
            this.completions.resolve_import(match[1], true);
            this.parse();
        }
    
        match = line.match(/\s*\/\*/);
        if (match) {
            this.state = State.MultilineComment;
            this.scratch = [];

            this.consume_multiline_comment(line);
            this.parse();
        }

        match = line.match(/^\s*\/\//);
        if (match) {
            if (this.lines[0] && this.lines[0].match(/^\s*\/\//)) {
                this.state = State.MultilineComment;
                this.scratch = [];

                this.consume_multiline_comment(line, true);
                this.parse();
            }
        }
        this.parse();
    }

    consume_multiline_comment(current_line: string, use_line_comment: boolean = false) {
        if (typeof current_line === 'undefined') {
            return; // EOF
        }

        let match: any = (use_line_comment) ? !/^\s*\/\//.test(current_line) : /\*\//.test(current_line);
        if (match) {
            if (this.state == State.DocComment) {
                if (use_line_comment) {
                    this.read_function(current_line);
                } else {
                    this.read_function(this.lines.shift());
                }
            }

            this.state = State.None;
            this.parse();
        } else {
            if (!use_line_comment) {
                match = current_line.match(/^\s*\*\s*@(?:param|return)\s*([A-Za-z_\.][A-Za-z0-9_\.]*)\s*(.*)/);
            } else {
                match = current_line.match(/^\s*\/\/\s*@(?:param|return)\s*([A-Za-z_\.][A-Za-z0-9_\.]*)\s*(.*)/);
            }

            if (match) {
                this.state = State.DocComment;
            }

            this.scratch.push(current_line);

            this.consume_multiline_comment(this.lines.shift(), use_line_comment);
        }
    }

    read_function(line: string) {
        // TODO: Support multiline function definitions
        let match = line.match(/\s*(?:native|stock|public)\s*([^\s]+)\s*([A-Za-z_].*)/);
        if (match) {
            let description = this.scratch.filter((line) => {
                return /^\s*\*\s+([^@].*)/.test(line) || /^\s*\/\/\s+([^@].*)/.test(line);
            }).map((line) => {
                return line.replace(/^\s*\*\s+/, "").replace(/^\s*\/\/\s+/, "");
            }).join(' ');
            
            const paramRegex = /@param\s+([A-Za-z0-9_\.]+)\s+(.*)/;
            let params = this.scratch.filter((line) => {
                return paramRegex.test(line);
            }).map((line) => {
                let match = paramRegex.exec(line);
                return {label: match[1], documentation: match[2]};
            });
            let name_match = match[2].match(/^([A-Za-z_][A-Za-z0-9_]*)/);
            this.completions.add(name_match[1], new FunctionCompletion(name_match[1], match[2], description, params));
        }

        this.state = State.None;
        this.parse();
    }
}