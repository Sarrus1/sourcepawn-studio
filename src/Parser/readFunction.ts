import { Range } from "vscode";

import { Parser } from "./spParser";
import { MethodItem } from "../Backend/Items/spMethodItem";
import { FunctionItem } from "../Backend/Items/spFunctionItem";
import { State } from "./stateEnum";
import { parseDocComment } from "./parseDocComment";
import {
  parentCounter,
  getParenthesisCount,
  isSingleLineFunction,
  getParamsFromDeclaration,
} from "./utils";
import { isControlStatement } from "../Providers/spDefinitionProvider";
import { addVariableItem } from "./addVariableItem";
import { MethodMapItem } from "../Backend/Items/spMethodmapItem";
import { EnumStructItem } from "../Backend/Items/spEnumStructItem";

export function readFunction(
  parser: Parser,
  match: RegExpMatchArray,
  line: string
): void {
  if (
    isControlStatement(line) ||
    /\bfunction\b/.test(match[1]) ||
    parser.state.includes[State.Function]
  ) {
    return;
  }
  if (parser.state.includes(State.Property)) {
    if (!/;\s*$/.test(line)) {
      parser.state.push(State.Function);
    }
    return;
  }
  if (line === undefined) {
    return;
  }
  let newSyntaxRe: RegExp = /^\s*(?:(?:stock|public|native|forward|static)\s+)*(?:(\w*(?:\s*\[[\w \+\-\*]*\]\s*)?)\s+)?(\w*)\s*\((.*(?:\)|,|{))?\s*/;
  match = line.match(newSyntaxRe);
  if (!match) {
    match = line.match(
      /^\s*(?:(?:static|native|stock|public|forward)\s+)*(?:(\w+)\s*:)?\s*(\w*)\s*\(([^\)]*(?:\)?))(?:\s*)(?:\{?)(?:\s*)(?:[^\;\s]*);?\s*$/
    );
  }
  let isMethod: boolean =
    parser.state.includes(State.Methodmap) ||
    parser.state.includes(State.EnumStruct);

  // We can't declare a function inside a function, this is a call.
  // cancel the parsing
  if (!match || parser.state.includes(State.Function)) {
    return;
  }

  let { description, params } = parseDocComment(parser);
  let nameMatch = match[2];
  // Stop if it's a macro being called
  if (parser.macroArr.length > 0) {
    let tmpStr = "";
    if (parser.macroArr.length > 1) {
      tmpStr = `\\b(?:${parser.macroArr.join("|")})\\b`;
    } else {
      tmpStr = `\\b(?:${parser.macroArr[0]})\\b`;
    }
    let macroRe = new RegExp(tmpStr);
    if (macroRe.test(nameMatch)) {
      // Check if we are still in the conditionnal of the control statement
      // for example, an if statement's conditionnal can span over several lines
      // and call functions
      let parenthesisNB = parentCounter(line);
      let lineCounter = 0;
      let iter = 0;
      while (parenthesisNB !== 0 && iter < 100) {
        iter++;
        line = parser.lines[lineCounter];
        lineCounter++;
        parenthesisNB += parentCounter(line);
      }
      // Now we test if the statement uses brackets, as short code blocks are usually
      // implemented without them.
      if (!/\{\s*$/.test(line)) {
        // Test the next line if we didn't match
        if (!/^\s*\{/.test(parser.lines[lineCounter])) {
          return;
        }
      }
      parser.state.push(State.Macro);
      return;
    }
  }

  let item: MethodItem | FunctionItem;
  if (isMethod) {
    item = new MethodItem(
      parser.fileItems.get(parser.state_data.name) as
        | MethodMapItem
        | EnumStructItem,
      nameMatch,
      "",
      description,
      params,
      "",
      parser.filePath,
      undefined,
      parser.IsBuiltIn,
      undefined,
      parser.deprecated
    );
  } else {
    item = new FunctionItem(
      nameMatch,
      "",
      description,
      params,
      parser.filePath,
      parser.IsBuiltIn,
      undefined,
      undefined,
      undefined,
      parser.deprecated
    );
  }
  const oldLastFunc = parser.lastFunc;
  parser.lastFunc = item;
  parser.lastFuncLine = parser.lineNb;

  let type = match[1];
  let paramsMatch = match[3] === undefined ? "" : match[3];
  addParamsDef(parser, paramsMatch, line);
  // Iteration safety in case something goes wrong
  let maxiter = 0;
  let matchEndRegex: RegExp = /(\{|\;)\s*(?:(?:\/\/|\/\*)(?:.*))?$/;
  let isNativeOrForward = /\bnative\b|\bforward\b/.test(match[0]);
  let matchEnd = matchEndRegex.test(line);
  let pCount = getParenthesisCount(line);
  let matchLastParenthesis = pCount === 0;
  let range = parser.makeDefinitionRange(nameMatch, line, true);

  while (
    !(matchLastParenthesis && matchEnd) &&
    line !== undefined &&
    maxiter < 20
  ) {
    maxiter++;
    line = parser.lines.shift();
    parser.lineNb++;
    if (line === undefined) {
      return;
    }
    if (!matchLastParenthesis) {
      addParamsDef(parser, line, line);
      paramsMatch += line;
      pCount += getParenthesisCount(line);
      matchLastParenthesis = pCount === 0;
    }
    if (!matchEnd) {
      if (matchLastParenthesis && /\,\s*$/.test(paramsMatch)) {
        // If the statement ends with a comma, we are in an array declaration
        parser.lastFuncLine = -1;
        parser.lastFunc = oldLastFunc;
        return;
      }
      matchEnd = matchEndRegex.test(line);
    }
  }
  if (!matchEnd) {
    parser.lastFuncLine = -1;
    parser.lastFunc = oldLastFunc;
    return;
  }
  let endSymbol = line.match(matchEndRegex);
  if (endSymbol === null) {
    parser.lastFuncLine = -1;
    parser.lastFunc = oldLastFunc;
    return;
  }

  if (isNativeOrForward) {
    if (endSymbol[1] === "{") {
      parser.lastFuncLine = -1;
      parser.lastFunc = oldLastFunc;
      return;
    }
  } else {
    if (endSymbol[1] === ";" || endSymbol[1] === ",") {
      parser.lastFuncLine = -1;
      parser.lastFunc = oldLastFunc;
      return;
    } else if (!isSingleLineFunction(line)) {
      parser.state.push(State.Function);
    }
  }
  // Treat differently if the function is declared on multiple lines
  paramsMatch = /\)\s*(?:\{|;)?\s*$/.test(match[0])
    ? match[0]
    : match[0].replace(/\(.*\s*$/, "(") +
      paramsMatch.replace(/\s*\w+\s*\(\s*/g, "").replace(/\s+/gm, " ");
  if (params.length === 0) {
    params = getParamsFromDeclaration(paramsMatch);
  }

  if (isMethod) {
    let fullRange = new Range(
      range.start.line,
      match.index,
      parser.lineNb,
      match.index + match[0].length
    );
    item.detail = paramsMatch
      .replace(/;\s*$/g, "")
      .replace(/{\s*$/g, "")
      .trim();
    item.params = params;
    item.type = type;
    item.range = range;
    item.fullRange = fullRange;
    parser.fileItems.set(nameMatch + parser.state_data.name, item);
    parser.deprecated = undefined;
    return;
  }

  let fullRange: Range;
  if (isNativeOrForward) {
    let end = range.start.line === parser.lineNb ? line.length : 0;
    fullRange = new Range(range.start.line, match.index, parser.lineNb, end);
  } else {
    fullRange = new Range(
      range.start.line,
      match.index,
      parser.lineNb,
      match.index + match[0].length
    );
  }

  item.detail = paramsMatch.replace(/;\s*$/g, "").replace(/{\s*$/g, "").trim();
  item.params = params;
  item.range = range;
  item.type = type;
  item.fullRange = fullRange;
  parser.fileItems.set(nameMatch, item);
  parser.lastFunc = item;

  parser.deprecated = undefined;
}

export function addParamsDef(parser: Parser, params: string, line: string) {
  let match_variable: RegExpExecArray;
  let match_variables: RegExpExecArray[] = [];
  let re = /\s*(?:(?:const|static)\s+)?(?:(\w+)(?:\s*(?:\[(?:[A-Za-z_0-9+* ]*)\])?\s+|\s*\:\s*))?(\w+)(?:\[(?:[A-Za-z_0-9+* ]*)\])?(?:\s*=\s*(?:[^,]+))?/g;
  do {
    match_variable = re.exec(params);
    if (match_variable) {
      match_variables.push(match_variable);
    }
  } while (match_variable);

  for (let variable of match_variables) {
    let variable_completion = variable[2].match(
      /(?:\s*)?([A-Za-z_,0-9]*)(?:(?:\s*)?(?:=(?:.*)))?/
    )[1];
    if (!parser.IsBuiltIn) {
      addVariableItem(parser, variable_completion, line, variable[1], true);
    }
  }
}
