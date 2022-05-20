export enum Quote {
  None,
  Single,
  Double,
}

export interface LineState {
  openedQuote: Quote;
  blockComment: boolean;
}

export enum ConditionState {
  None,
  if,
  else,
  elseIf,
}

export class PreProcessor {
  lines: string[];
  lineNb: number;
  line: string;
  preprocessedLines: string;
  defines: Map<string, string>;
  conditionState: ConditionState;
  conditionWasActivated: boolean;
  skipLine: boolean;

  constructor(lines: string[]) {
    this.lines = lines;
    this.lineNb = 0;
    this.defines = new Map();
    this.conditionState = ConditionState.None;
    this.conditionWasActivated = false;
    this.preprocessedLines = "";
  }

  private addLine(line: string) {
    this.preprocessedLines += line + "\n";
  }

  public preProcess(): string {
    for (let line of this.lines) {
      let match = line.match(/^\s*#define\s+([A-Za-z_]\w*)[^\S\r\n]*/);

      if (match) {
        this.handleDefine(match, line);
        continue;
      }

      match = line.match(/^\s*#if/);

      if (match) {
        this.handleIf(match, line, ConditionState.if);
        continue;
      }

      match = line.match(/^\s*#elseif/);

      if (match) {
        this.handleIf(match, line, ConditionState.elseIf);
        continue;
      }

      match = line.match(/^\s*#else/);

      if (match) {
        this.handleElse(line);
        continue;
      }

      match = line.match(/^\s*#endif/);

      if (match) {
        this.conditionState = ConditionState.None;
        this.conditionWasActivated = false;
        this.skipLine = false;
        this.addLine("");
        continue;
      }

      if (this.skipLine) {
        this.addLine("");
      } else {
        this.addLine(line);
      }
    }
    return this.preprocessedLines;
  }

  private handleDefine(match: RegExpMatchArray, line: string) {
    // Add the line no matter what, to get define autocompletion.
    this.addLine(line);

    const defineValSt = match.index + match[0].length;
    if (line.length <= defineValSt) {
      return;
    }
    const lineState: LineState = {
      openedQuote: Quote.None,
      blockComment: false,
    };
    let value = "";
    for (let i = defineValSt; i < line.length; i++) {
      if (!lineState.blockComment && lineState.openedQuote === Quote.None) {
        // No comment, no string.
        if (i + 1 >= line.length) {
          // Too short to continue, append value and exit.
          // TODO: Check if multiline.
          value += line[i];
          break;
        }
        if (line[i] === "/" && line[i] === "/") {
          // Start of a line comment, exit.
          break;
        }
        if (line[i] === "/" && line[i] === "*") {
          // Start of a block comment, check if it spans the whole line.
          // TODO: Implement logic.
          break;
        }
        if (line[i] === '"') {
          lineState.openedQuote = Quote.Double;
        } else if (line[i] === "'") {
          lineState.openedQuote = Quote.Single;
        }
        value += line[i];
      }
      if (lineState.openedQuote === Quote.Single) {
        // Single quote string.
        if (i + 1 >= line.length) {
          if (line[i] === "\\") {
            // Line continuation.
          }
          value += line[i];
        }
        if (line[i] === "'" && line[i - 1] !== "\\") {
          lineState.openedQuote = Quote.None;
        }
        value += line[i];
        continue;
      }
      if (lineState.openedQuote === Quote.Double) {
        // Single quote string.
        if (i + 1 >= line.length) {
          if (line[i] === "\\") {
            // Line continuation.
          }
          value += line[i];
        }
        if (line[i] === '"' && line[i - 1] !== "\\") {
          lineState.openedQuote = Quote.None;
        }
        value += line[i];
      }
    }

    this.defines.set(match[1], value.trim());
  }

  private handleIf(
    match: RegExpMatchArray,
    line: string,
    state: ConditionState
  ) {
    this.conditionState = state;
    if (this.conditionWasActivated) {
      this.skipLine = true;
      this.addLine("");
      return;
    }
    let condition = line.slice(match.index + match[0].length);
    const matches = condition.match(/[A-Za-z_]\w*/g);
    if (matches) {
      // FIXME: Handle "defined".
      for (let subMatch of matches) {
        let define = this.defines.get(subMatch);
        if (define !== undefined) {
          condition = condition.replace(subMatch, define);
        }
      }
    }
    const evaluation = eval(condition);
    if (evaluation) {
      this.conditionWasActivated = true;
      this.skipLine = false;
    }
    this.addLine("");
    return;
  }

  private handleElse(line: string) {
    this.conditionState = ConditionState.else;
    if (this.conditionWasActivated) {
      this.skipLine = true;
      this.addLine("");
      return;
    }
    this.conditionWasActivated = true;
    this.skipLine = false;
    this.addLine(line);
  }
}
