import {
  CompletionItem,
  CompletionItemKind,
  Position,
  SnippetString,
  Range,
  CompletionItemProvider,
  TextDocument,
  CancellationToken,
} from "vscode";

import { getParamsFromDeclaration } from "./spParser";

const indentSize: number = 5;

class SpDocCompletionItem extends CompletionItem {
  constructor(position: Position, FunctionDesc: string[], indent: string) {
    super("/** */", CompletionItemKind.Text);
    let snippet = new SnippetString();
    let max = getMaxLength(FunctionDesc);
    snippet.appendText(`${indent}/**\n ${indent}* `);
    snippet.appendPlaceholder("Description");
    //snippet.appendText("\n *");
    this.appendTextSnippet(snippet, "", indent);
    for (let arg of FunctionDesc) {
      this.appendTextSnippet(
        snippet,
        "@param " + arg + " ".repeat(getSpaceLength(arg, max)),
        indent
      );
      snippet.appendPlaceholder("Param description");
    }
    this.appendTextSnippet(
      snippet,
      "@return " + " ".repeat(getSpaceLengthReturn(max)),
      indent
    );
    snippet.appendPlaceholder("Return description");
    snippet.appendText(`"\n${indent} */`);
    this.insertText = snippet;
    let start: Position = new Position(position.line, 0);
    let end: Position = new Position(position.line, 0);
    this.range = new Range(start, end);
  }

  private appendTextSnippet(
    snippet: SnippetString,
    text: string,
    indent: string
  ): void {
    snippet.appendText(`\n${indent} * ${text}`);
  }
}

export class JsDocCompletionProvider implements CompletionItemProvider {
  public async provideCompletionItems(
    document: TextDocument,
    position: Position,
    token: CancellationToken
  ): Promise<CompletionItem[] | undefined> {
    if (!document) {
      return undefined;
    }
    let { params, indent } = getFullParams(document, position);

    let functionDesc = getParamsFromDeclaration(params).map((e) => e.label);
    if (functionDesc == []) {
      return undefined;
    }
    let DocCompletionItem = new SpDocCompletionItem(
      position,
      functionDesc,
      indent
    );
    return [DocCompletionItem];
  }
}

function getMaxLength(arr: string[]): number {
  let max: number = 0;
  for (let str of arr) {
    if (str.length > max) max = str.length;
  }
  return max;
}

function getSpaceLength(str: string, max: number): number {
  return max + indentSize - str.length;
}

function getSpaceLengthReturn(max): number {
  return max + indentSize - 1;
}

function getFullParams(document: TextDocument, position: Position) {
  const lines = document.getText().split("\n");
  let lineNB = position.line + 1;
  let line = lines[lineNB];
  let newSyntaxRe: RegExp = /^(\s)*(?:(?:stock|public|native|forward|static)\s+)*(?:(\w*)\s+)?(\w*)\s*\((.*(?:\)|,|{))\s*/;
  let match: RegExpMatchArray = line.match(newSyntaxRe);
  if (!match) {
    match = line.match(
      /^(\s)*(?:(?:static|native|stock|public|forward)\s+)*(?:(\w+)\s*:)?\s*(\w*)\s*\(([^\)]*(?:\)?))(?:\s*)(?:\{?)(?:\s*)(?:[^\;\s]*);?\s*$/
    );
  }
  let isNativeOrForward = /\bnative\b|\bforward\b/.test(match[0]);
  let paramsMatch = match[4];
  let matchEndRegex: RegExp = /(\{|\;)\s*(?:(?:\/\/|\/\*)(?:.*))?$/;
  let matchEnd = matchEndRegex.test(line);
  let matchLastParenthesis = /\)/.test(paramsMatch);
  let iter = 0;
  while (
    !(matchLastParenthesis && matchEnd) &&
    typeof line != "undefined" &&
    iter < 20
  ) {
    iter++;
    lineNB++;
    line = lines[lineNB];

    if (!matchLastParenthesis) {
      paramsMatch += line;
      matchLastParenthesis = /\)/.test(paramsMatch);
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
    if (endSymbol[1] === "{") return;
  } else if (endSymbol[1] === ";") {
    return;
  }
  // Treat differently if the function is declared on multiple lines
  paramsMatch = /\)\s*(?:\{|;)?\s*$/.test(match[0])
    ? match[0]
    : match[0].replace(/\(.*\s*$/, "(") +
      paramsMatch.replace(/\s*\w+\s*\(\s*/g, "").replace(/\s+/gm, " ");
  return { params: paramsMatch, indent: match[1] == undefined ? "" : match[1] };
}
