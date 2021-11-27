import { Parser } from "./spParser";
import { MethodItem, FunctionItem } from "../Providers/spItems";
import { State } from "./stateEnum";
import { Range } from "vscode";
import { searchForDefinesInString } from "./searchForDefinesInString";
import { parseDocComment } from "./parseDocComment";
import {
  parentCounter,
  getParenthesisCount,
  isSingleLineFunction,
  getParamsFromDeclaration,
} from "./utils";
import { isControlStatement } from "../Providers/spDefinitions";
import { addVariableItem } from "./addVariableItem";

export function readFunction(
  parser: Parser,
  match: RegExpMatchArray,
  line: string
): void {
  if (isControlStatement(line) || /\bfunction\b/.test(match[1])) {
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
  if (parser.state[parser.state.length - 1] === State.Function) {
    return;
  }
  if (match) {
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

    let lineMatch = parser.lineNb;
    let type = match[1];
    let paramsMatch = match[3] === undefined ? "" : match[3];
    addParamsDef(parser, paramsMatch, nameMatch, line);
    // Iteration safety in case something goes wrong
    let maxiter = 0;
    let matchEndRegex: RegExp = /(\{|\;)\s*(?:(?:\/\/|\/\*)(?:.*))?$/;
    let isNativeOrForward = /\bnative\b|\bforward\b/.test(match[0]);
    let matchEnd = matchEndRegex.test(line);
    let pCount = getParenthesisCount(line);
    let matchLastParenthesis = pCount === 0;
    let range = parser.makeDefinitionRange(nameMatch, line);

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
        addParamsDef(parser, line, nameMatch, line);
        searchForDefinesInString(parser, line);
        paramsMatch += line;
        pCount += getParenthesisCount(line);
        matchLastParenthesis = pCount === 0;
      }
      if (!matchEnd) {
        if (matchLastParenthesis && /\,\s*$/.test(paramsMatch)) {
          // If the statement ends with a comma, we are in an array declaration
          return;
        }
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
    } else {
      if (endSymbol[1] === ";" || endSymbol[1] === ",") {
        return;
      } else if (!isSingleLineFunction(line)) {
        parser.state.push(State.Function);
      }
    }
    parser.lastFuncLine = lineMatch;
    parser.lastFuncName = nameMatch;
    // Treat differently if the function is declared on multiple lines
    paramsMatch = /\)\s*(?:\{|;)?\s*$/.test(match[0])
      ? match[0]
      : match[0].replace(/\(.*\s*$/, "(") +
        paramsMatch.replace(/\s*\w+\s*\(\s*/g, "").replace(/\s+/gm, " ");
    if (params.length === 0) {
      params = getParamsFromDeclaration(paramsMatch);
    }
    if (isMethod) {
      let fullRange: Range;
      if (isNativeOrForward) {
        let end = range.start.line === parser.lineNb ? line.length : 0;
        fullRange = new Range(range.start.line, 0, parser.lineNb, end);
      }
      parser.completions.add(
        nameMatch + parser.state_data.name,
        new MethodItem(
          parser.state_data.name,
          nameMatch,
          paramsMatch.replace(/;\s*$/g, "").replace(/{\s*$/g, "").trim(),
          description,
          params,
          type,
          parser.file,
          range,
          parser.IsBuiltIn,
          fullRange
        )
      );
      return;
    }
    // For small files, the parsing is too fast and functions get overwritten by their own calls.
    // If we define a function somewhere, we won't redefine it elsewhere. We can safely ignore it.
    if (parser.completions.get(nameMatch)) {
      return;
    }
    let fullRange: Range;
    if (isNativeOrForward) {
      let end = range.start.line === parser.lineNb ? line.length : 0;
      fullRange = new Range(range.start.line, 0, parser.lineNb, end);
    }
    parser.completions.add(
      nameMatch,
      new FunctionItem(
        nameMatch,
        paramsMatch.replace(/;\s*$/g, "").replace(/{\s*$/g, "").trim(),
        description,
        params,
        parser.file,
        parser.IsBuiltIn,
        range,
        type,
        fullRange
      )
    );
  }
}

export function addParamsDef(
  parser: Parser,
  params: string,
  funcName: string,
  line: string
) {
  let match_variable: RegExpExecArray;
  let match_variables: RegExpExecArray[] = [];
  let re = /\s*(?:(?:const|static)\s+)?(?:(\w+)(?:\s*(?:\[(?:[A-Za-z_0-9+* ]*)\])?\s+|\s*\:\s*))?(\w+)(?:\[(?:[A-Za-z_0-9+* ]*)\])?(?:\s*=\s*(?:[^,]+))?/g;
  while ((match_variable = re.exec(params)) != null) {
    match_variables.push(match_variable);
  }
  for (let variable of match_variables) {
    let variable_completion = variable[2].match(
      /(?:\s*)?([A-Za-z_,0-9]*)(?:(?:\s*)?(?:=(?:.*)))?/
    )[1];
    if (!parser.IsBuiltIn) {
      addVariableItem(
        parser,
        variable_completion,
        line,
        variable[1],
        funcName,
        true
      );
    }
  }
}
