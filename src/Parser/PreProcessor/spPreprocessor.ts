import { Diagnostic, DiagnosticSeverity, Range } from "vscode";
import { URI } from "vscode-uri";
import evaluate from "safe-evaluate-expression";

import { FileItem } from "../../Backend/spFilesRepository";
import { ItemsRepository } from "../../Backend/spItemsRepository";
import { isIncludeSelfFile } from "../utils";
import { getAllDefines } from "../../Backend/spItemsGetters";
import { preDiagnostics } from "../../Providers/Linter/compilerDiagnostics";
import { readFileSync } from "fs";

export enum ParseState {
  None,
  SingleQuote,
  DoubleQuote,
  BlockComment,
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
  conditionState: ConditionState;
  conditionWasActivated: boolean[];
  skipLine: boolean;
  fileItem: FileItem;
  uri: URI;
  itemsRepo: ItemsRepository;
  range: Range | undefined;
  diagnostics: Diagnostic[];

  constructor(lines: string[], fileItem: FileItem, itemsRepo: ItemsRepository) {
    this.lines = lines;
    this.lineNb = -1;
    this.conditionState = ConditionState.None;
    this.conditionWasActivated = [];
    this.preprocessedLines = "";
    this.fileItem = fileItem;
    this.itemsRepo = itemsRepo;
    this.uri = URI.parse(fileItem.uri);
    preDiagnostics.delete(this.uri);
    this.diagnostics = [];
  }

  private getLine(): string {
    return this.lines[this.lineNb];
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
    for (this.lineNb = 0; this.lineNb < this.lines.length; this.lineNb++) {
      let match = this.getLine().match(
        /^\s*#define\s+([A-Za-z_]\w*)[^\S\r\n]+/
      );

      if (match) {
        this.handleDefine(match, this.getLine());
        continue;
      }

      match = this.getLine().match(/^\s*#include\s+<([A-Za-z0-9\-_\/.]+)>/);
      if (match) {
        this.handleInclude(match);
        continue;
      }
      match = this.getLine().match(/^\s*#include\s+"([A-Za-z0-9\-_\/.]+)"/);
      if (match) {
        this.handleInclude(match);
        continue;
      }

      match = this.getLine().match(/^\s*#if/);

      if (match) {
        this.handleIf(match, this.getLine(), ConditionState.if);
        continue;
      }

      match = this.getLine().match(/^\s*#elseif/);

      if (match) {
        this.handleIf(match, this.getLine(), ConditionState.elseIf);
        continue;
      }

      match = this.getLine().match(/^\s*#else/);

      if (match) {
        this.handleElse(this.getLine());
        continue;
      }

      match = this.getLine().match(/^\s*#emit/);

      if (match) {
        this.addLine("");
        continue;
      }

      match = this.getLine().match(/^\s*#endif/);

      if (match) {
        this.conditionState = ConditionState.None;
        this.conditionWasActivated.pop();
        this.skipLine = false;
        this.addLine("");
        continue;
      }

      if (this.skipLine) {
        this.addLine("");
      } else {
        this.addLine(this.getLine());
      }
    }
    preDiagnostics.set(this.uri, this.diagnostics);
    return this.preprocessedLines;
  }

  private handleDefine(match: RegExpMatchArray, line: string) {
    let emptyLinesToAdd = 0;
    let escapedChar = false;
    // Add the line no matter what, to get the define in the AST.
    let lineToAdd = line.replace(/\\(?:\r)$/, "").trim();
    // this.addLine(line);

    const defineValSt = match.index + match[0].length;
    if (line.length <= defineValSt) {
      this.addLine(lineToAdd);
      return;
    }
    let state = ParseState.None;

    let value = "";
    let i = defineValSt;
    loop: for (i; i < line.length; i++) {
      switch (state) {
        case ParseState.None:
          // No comment, no string.
          if (/^\\(?:\r)$/.test(line.slice(i))) {
            // Line termination sequence reached.
            this.lineNb++;
            i = -1;
            line = this.getLine();
            if (line === undefined) {
              break loop;
            }
            lineToAdd += line.replace(/\\(?:\r)$/, "").trim();
            emptyLinesToAdd++;
          }
          if (i == line.length - 1) {
            value += line[i];
            break loop;
          }
          if (line[i] === "/" && line[i + 1] === "/") {
            // Start of a line comment, exit.
            break loop;
          }
          if (line[i] === "/" && line[i + 1] === "*") {
            // Start of a block comment, check if it spans the whole line.
            value += "/*";
            i++;
            state = ParseState.BlockComment;
            continue loop;
          }
          if (line[i] === '"') {
            state = ParseState.DoubleQuote;
          } else if (line[i] === "'") {
            state = ParseState.SingleQuote;
          }
          value += line[i];
          break;
        case ParseState.BlockComment:
          // In a block comment.
          if (i == line.length - 1) {
            // EOL, go to the next line.
            this.lineNb++;
            i = -1;
            line = this.getLine();
            if (line === undefined) {
              break loop;
            }
            // Add a space to replace the line break.
            lineToAdd += " " + line.replace(/\\(?:\r)$/, "").trim();
            emptyLinesToAdd++;
            continue loop;
          }
          if (line[i] === "*" && line[i + 1] === "/") {
            // End of the block comment.
            state = ParseState.None;
            i++;
            value += "*/";
            continue loop;
          }
          value += line[i];
          break;
        case ParseState.SingleQuote:
          // Single quote string.
          if (!escapedChar) {
            if (line[i] === "\\") {
              escapedChar = true;
            } else {
              if (line[i] === "'") {
                state = ParseState.None;
              }
              value += line[i];
            }
            continue;
          }
          escapedChar = false;
          if (/(?:\r)?$/.test(line.slice(i))) {
            // Line continuation
            this.lineNb++;
            i = -1;
            line = this.getLine();
            if (line === undefined) {
              break loop;
            }
            // Add a space to replace the line break.
            lineToAdd += line.replace(/\\(?:\r)$/, "").trim();
            emptyLinesToAdd++;
            continue loop;
          } else {
            value += line[i];
          }
          break;
        case ParseState.DoubleQuote:
          // Double quote string.
          if (!escapedChar) {
            if (line[i] === "\\") {
              escapedChar = true;
            } else {
              if (line[i] === '"') {
                state = ParseState.None;
              }
              value += line[i];
            }
            continue;
          }
          escapedChar = false;
          if (/(?:\r)?$/.test(line.slice(i))) {
            // Line continuation
            this.lineNb++;
            i = -1;
            line = this.getLine();
            if (line === undefined) {
              break loop;
            }
            // Add a space to replace the line break.
            lineToAdd += line.replace(/\\(?:\r)$/, "").trim();
            emptyLinesToAdd++;
            continue loop;
          } else {
            value += line[i];
          }
      }
    }
    this.addLine(lineToAdd);
    for (let i = 0; i < emptyLinesToAdd; i++) {
      this.addLine("");
    }
    value = value.trim();
    this.fileItem.defines.set(match[1], value);
  }

  private handleIf(
    match: RegExpMatchArray,
    line: string,
    state: ConditionState
  ) {
    this.conditionState = state;
    if (
      state === ConditionState.elseIf &&
      this.conditionWasActivated[this.conditionWasActivated.length - 1]
    ) {
      this.skipLine = true;
      this.addLine("");
      return;
    }
    const defines = getAllDefines(this.itemsRepo, this.uri, this.fileItem);
    let condition = line.slice(match.index + match[0].length).trim();
    const matches = condition.match(/\b[A-Za-z_]\w*\b/g);
    if (matches) {
      for (let i = 0; i < matches.length; i++) {
        // Handle "defined"
        if (matches[i] === "defined") {
          if (i + 1 < matches.length && defines.has(matches[i + 1])) {
            condition = condition.replace(
              RegExp(`defined\\s*${matches[i + 1]}`),
              "true"
            );
          } else {
            condition = condition.replace(
              RegExp(`defined\\s*${matches[i + 1]}`),
              "false"
            );
          }
        }
        const define = defines.get(matches[i]);
        if (define !== undefined) {
          condition = condition.replace(matches[i], define);
        }
      }
    }
    let evaluation = false;
    try {
      evaluation = evaluate(condition);
    } catch (err) {
      // TODO: Make the range more precise.
      const range = new Range(this.lineNb, 0, this.lineNb, line.length);
      this.diagnostics.push(
        new Diagnostic(
          range,
          "Invalid expression. " + err.message,
          DiagnosticSeverity.Error
        )
      );
    }

    if (evaluation) {
      this.conditionWasActivated.push(true);
      this.skipLine = false;
    } else {
      this.conditionWasActivated.push(false);
      this.skipLine = true;
    }
    this.addLine("");
    return;
  }

  private handleElse(line: string) {
    this.conditionState = ConditionState.else;
    if (this.conditionWasActivated[this.conditionWasActivated.length - 1]) {
      this.skipLine = true;
      this.addLine("");
      return;
    }
    this.conditionWasActivated.push(true);
    this.skipLine = false;
    this.addLine(line);
  }

  private handleInclude(match: RegExpMatchArray) {
    const includePath = match[1];
    const filePath = this.uri.fsPath;
    if (isIncludeSelfFile(filePath, includePath)) {
      return;
    }
    const resolved = this.fileItem.resolveImport(
      includePath,
      this.itemsRepo.documents,
      filePath,
      new Range(
        this.lineNb,
        match.index,
        this.lineNb,
        match.index + match[1].length
      )
    );
    this.addLine("");
    if (resolved === undefined || this.itemsRepo.fileItems.has(resolved)) {
      return;
    }
    const uri = URI.parse(resolved);
    const fileItem: FileItem = new FileItem(uri.toString());
    this.itemsRepo.documents.set(uri.toString(), false);
    this.itemsRepo.fileItems.set(uri.toString(), fileItem);
    try {
      const text = readFileSync(uri.fsPath).toString();
      const preprocessor = new PreProcessor(
        text.split("\n"),
        fileItem,
        this.itemsRepo
      );
      fileItem.text = preprocessor.preProcess();
    } catch (err) {
      console.error(err);
    }
  }
}
