import { CompletionRepository, FunctionCompletion, DefineCompletion } from './completions';
import * as fs from 'fs';
import * as uuid from 'uuid';

export function parse_file(file: string, completions: CompletionRepository) {
    fs.readFile(file, "utf-8", (err, data) => {
        let lines = data.split("\n");
        let parser = new Parser(lines, completions);

        parser.parse();
    });
}

enum State {
    None,
    MultilineComment,
    DocComment,
    Enum,
}

class Parser {
    lines: string[];
    completions: CompletionRepository;
    state: State;
    scratch: any;

    constructor(lines: string[], completions: CompletionRepository) {
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
            let id = uuid();
            this.completions.add(id, new DefineCompletion(id, match[1]));
        }
    
        match = line.match(/\s*\/\*/);
        if (match) {
            this.state = State.MultilineComment;
            this.scratch = [];

            this.consume_multiline_comment(line);
        }
        this.parse();
    }

    consume_multiline_comment(current_line: string) {
        let match = current_line.match(/\*\//);
        if (match) {
            if (this.state == State.DocComment) {
                this.read_function(this.lines.shift());
            }

            this.state = State.None;
            this.parse();
        } else {
            match = current_line.match(/^\s*\*\s*@(?:param|return)\s*([A-Za-z_\.][A-Za-z0-9_\.]*)\s*(.*)/);
            if (match) {
                this.state = State.DocComment;
            }

            this.scratch.push(current_line);

            this.consume_multiline_comment(this.lines.shift());
        }
    }

    read_function(line: string) {
        let match = line.match(/\s*(?:native|stock|public)\s*([^\s]+)\s*([A-Za-z_][A-Za-z0-9_]*)/);
        if (match) {
            let id = uuid();
            this.completions.add(id, new FunctionCompletion(id, match[2], this.scratch.join("\n")));
        }

        this.state = State.None;
        this.parse();
    }
}