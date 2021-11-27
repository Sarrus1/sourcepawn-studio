import { FunctionParam, SPItem } from "../Providers/spItems";
import { Range } from "vscode";
import { basename } from "path";
import { URI } from "vscode-uri";

export function purgeCalls(item: SPItem, file: string): void {
  let uri = URI.file(file);
  if (item.calls === undefined) return;
  item.calls = item.calls.filter((e) => {
    uri === e.uri;
  });
}

export function positiveRange(
  lineNb: number,
  start: number = 0,
  end: number = 0
): Range {
  lineNb = lineNb > 0 ? lineNb : 0;
  start = start > 0 ? start : 0;
  end = end > 0 ? end : 0;
  return new Range(lineNb, start, lineNb, end);
}

export function isIncludeSelfFile(file: string, include: string): boolean {
  let baseName: string = basename(file);
  let match = include.match(/(\w*)(?:.sp|.inc)?$/);
  if (match) {
    return baseName == match[1];
  }
  return false;
}

export function getParamsFromDeclaration(decl: string): FunctionParam[] {
  let match = decl.match(/\((.+)\)/);
  if (!match) {
    return [];
  }
  // Remove the leading and trailing parenthesis
  decl = match[1] + ",";
  let params: FunctionParam[] = [];
  let re = /\s*(?:(?:const|static)\s+)?(?:(\w+)(?:\s*(?:\[(?:[A-Za-z_0-9+* ]*)\])?\s+|\s*\:\s*))?(\w+)(?:\[(?:[A-Za-z_0-9+* ]*)\])?(?:\s*=\s*(?:[^,]+))?/g;
  let matchVariable;
  do {
    matchVariable = re.exec(decl);
    if (matchVariable) {
      params.push({ label: matchVariable[2], documentation: "" });
    }
  } while (matchVariable);
  return params;
}

export function isSingleLineFunction(line: string) {
  return /\{.*\}\s*$/.test(line);
}

export function parentCounter(line: string): number {
  let counter = 0;
  if (line == null) {
    return 0;
  }
  for (let char of line) {
    if (char === "(") {
      counter++;
    } else if (char === ")") {
      counter--;
    }
  }
  return counter;
}

export function getParenthesisCount(line: string): number {
  let pCount = 0;
  let inAString = false;
  if (line === undefined) {
    return pCount;
  }
  for (let i = 0; i < line.length; i++) {
    let char = line[i];
    if (char === "'" || char === '"') {
      inAString = !inAString;
    } else if (!inAString && char === "(") {
      pCount++;
    } else if (!inAString && char === ")") {
      pCount--;
    }
  }
  return pCount;
}
