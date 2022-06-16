import { Range } from "vscode";
import { basename } from "path";

import { FunctionParam } from "./interfaces";
import { SPItem } from "../Backend/Items/spItems";
import { ParserLocation, spParserArgs } from "./interfaces";
import * as TreeSitter from "web-tree-sitter";

export function purgeReferences(
  item: SPItem,
  file: string,
  range: Range
): void {
  if (item.references === undefined) {
    return;
  }
  item.references = item.references.filter(
    (e) => file !== e.uri.fsPath || !containsIfDefined(range, e.range)
  );
}

function containsIfDefined(range1: Range, range2: Range): boolean {
  if (range1 === undefined) {
    return true;
  }
  return range1.contains(range2);
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
  const baseName: string = basename(file);
  const match = include.match(/(\w*)(?:.sp|.inc)?$/);
  if (match) {
    return baseName == match[1];
  }
  return false;
}

export function getParamsFromDeclaration(decl: string): FunctionParam[] {
  const match = decl.match(/\((.+)\)/);
  if (!match) {
    return [];
  }
  // Remove the leading and trailing parenthesis
  decl = match[1] + ",";
  const params: FunctionParam[] = [];
  const re = /\s*(?:(?:const|static)\s+)?(?:(\w+)(?:\s*(?:\[(?:[A-Za-z_0-9+* ]*)\])?\s+|\s*\:\s*|\s*&?\s*))?(\w+)(?:\[(?:[A-Za-z_0-9+* ]*)\])?(?:\s*=\s*(?:[^,]+))?/g;
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

export function getParenthesisCount(line: string): number {
  let pCount = 0;
  let inAString = false;
  if (line === undefined) {
    return pCount;
  }
  for (let i = 0; i < line.length; i++) {
    const char = line[i];
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

/**
 * Convert a parsed location of the parser to a range.
 * @param  {ParserLocation} loc
 * @param  {spParserArgs} args?
 * @returns Range
 */
export function parsedLocToRange(
  loc: ParserLocation,
  args?: spParserArgs
): Range {
  let offset = 0;
  if (args !== undefined) {
    offset = args.offset;
  }
  return new Range(
    loc.start.line - 1 + offset,
    loc.start.column - 1,
    loc.end.line - 1 + offset,
    loc.end.column - 1
  );
}

/**
 * Get a guess of the next scope in a file by finding the next non indented "}".
 * @param  {string} txt  The text of the file as a string.
 * @param  {number} lineNb  The current line number.
 * @returns { txt: string; offset: number } | undefined
 */
export function getNextScope(
  txt: string,
  lineNb: number
): { txt: string; newOffset: number } | undefined {
  lineNb++;
  const lines = txt.split("\n");
  if (lineNb >= lines.length) {
    return { txt: undefined, newOffset: undefined };
  }
  while (lineNb < lines.length) {
    if (/^}/.test(lines[lineNb]) && lineNb + 1 < lines.length) {
      return { txt: lines.slice(lineNb + 1).join("\n"), newOffset: lineNb + 1 };
    }
    lineNb++;
  }
  return { txt: undefined, newOffset: undefined };
}

/**
 * Check if the token is the declaration of the plugin infos.
 * @param  {string} name  The id of the token.
 * @param  {SPItem|undefined} lastFunc  The current function scope.
 * @param  {SPItem|undefined} lastMMorES  The current Methodmap or Enum struct scope.
 * @returns boolean
 */
export function checkIfPluginInfo(
  name: string,
  lastFunc: SPItem | undefined,
  lastMMorES: SPItem | undefined
): boolean {
  if (lastFunc !== undefined || lastMMorES !== undefined) {
    return false;
  }
  return ["Plugin", "Extension", "PlVers", "SharedPlugin"].includes(name);
}

export function pointsToRange(
  startPos: TreeSitter.Point,
  endPos: TreeSitter.Point
): Range {
  return new Range(startPos.row, startPos.column, endPos.row, endPos.column);
}
