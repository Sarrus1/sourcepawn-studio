import { Range } from "vscode";
import { URI } from "vscode-uri";
import { parse, compile } from "subscript";

import { DefineItem } from "../../Backend/Items/spDefineItem";
import { FileItem } from "../../Backend/spFilesRepository";
import { ItemsRepository } from "../../Backend/spItemsRepository";
import { isIncludeSelfFile } from "../utils";

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
  fileItem: FileItem;
  itemsRepo: ItemsRepository;
  range: Range | undefined;

  constructor(lines: string[], fileItem: FileItem, itemsRepo: ItemsRepository) {
    this.lines = lines;
    this.lineNb = -1;
    this.defines = new Map();
    this.conditionState = ConditionState.None;
    this.conditionWasActivated = false;
    this.preprocessedLines = "";
    this.fileItem = fileItem;
    this.itemsRepo = itemsRepo;
  }

  private addLine(line: string) {
    if (
      !this.range ||
      (this.range.start.line <= this.lineNb &&
        this.range.end.line >= this.lineNb)
    ) {
      this.preprocessedLines += line + "\n";
    }
  }

  public preProcess(range?: Range | undefined): string {
    this.range = range;
    for (let line of this.lines) {
      this.lineNb++;
      let match = line.match(/^\s*#define\s+([A-Za-z_]\w*)[^\S\r\n]+/);

      if (match) {
        this.handleDefine(match, line);
        continue;
      }

      match = line.match(/^\s*#include\s+<([A-Za-z0-9\-_\/.]+)>/);
      if (match) {
        this.handleInclude(match, line);
        continue;
      }
      match = line.match(/^\s*#include\s+"([A-Za-z0-9\-_\/.]+)"/);
      if (match) {
        this.handleInclude(match, line);
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
    // Add the line no matter what, to get the define in the AST.
    this.addLine(line);

    const lineNb = this.lineNb;

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
    value = value.trim();
    this.defines.set(match[1], value);

    const index = line.search(match[1]);
    const range = new Range(lineNb, index, lineNb, index + match[1].length);
    const defineItem = new DefineItem(
      match[1],
      value,
      "",
      URI.parse(this.fileItem.uri).fsPath,
      range,
      false,
      range
    );
    this.fileItem.items.push(defineItem);
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
    const matches = condition.match(/\b[A-Za-z_]\w*\b/g);
    if (matches) {
      for (let i = 0; i < matches.length; i++) {
        // Handle "defined"
        if (matches[i] === "defined") {
          if (i + 1 < matches.length && this.defines.has(matches[i + 1])) {
            condition = condition.replace(
              RegExp(`defined\\s*${matches[i]}`),
              "true"
            );
          } else {
            condition = "false";
            break;
          }
        }
        let define = this.defines.get(matches[i]);
        if (define !== undefined) {
          condition = condition.replace(matches[i], define);
        }
      }
    }
    let evaluation = false;
    try {
      const tree = parse(condition);
      evaluation = compile(tree)();
    } catch (err) {
      // TODO: Add document diagnostic if there is an error here.
      console.error(condition);
    }

    if (evaluation) {
      this.conditionWasActivated = true;
      this.skipLine = false;
    } else {
      this.skipLine = true;
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

  private handleInclude(match: RegExpMatchArray, line: string) {
    const includePath = match[1];
    const filePath = URI.parse(this.fileItem.uri).fsPath;
    if (isIncludeSelfFile(filePath, includePath)) {
      return;
    }
    this.fileItem.resolveImport(
      includePath,
      this.itemsRepo.documents,
      filePath,
      new Range(
        this.lineNb,
        match.index,
        this.lineNb,
        match.index + match[1].length
      ),
      // FIXME: Always false isbuiltin
      false
    );
    this.addLine("");
  }
}
