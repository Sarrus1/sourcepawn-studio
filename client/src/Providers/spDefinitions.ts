import { TextDocument, Range, Position } from "vscode";
import { globalIdentifier } from "./spGlobalIdentifier";

export function GetLastFuncName(
  position: Position,
  document: TextDocument
): string {
  let lineNB = position.line;
  let re = /(?:static|native|stock|public|forward)?\s*(?:[a-zA-Z\-_0-9]:)?([^\s]+)\s*([A-Za-z_]*)\s*\(([^\)]*)(?:\)?)(?:\s*)(?:\{?)(?:\s*)(?:[^\;\s]*);?\s*$/;
  let text = document.getText().split("\n");
  let Match;
  let line: string;
  for (lineNB; lineNB > -1; lineNB--) {
    line = text[lineNB];
    if (line.match(/^\}/)) return globalIdentifier;
    Match = line.match(re);
    if (Match) {
      let match = line.match(
        /^\s*(?:(?:stock|public)\s+)*(?:(\w*)\s+)?(\w*)\s*\(([^]*)(?:\)|,|{)\s*$/
      );
      if (!match) {
        match = line.match(
          /^\s*(?:(?:forward|static|native)\s+)+(?:(\w*)\s+)?(\w*)\s*\(([^]*)(?:,|;)\s*$/
        );
      }
      if (match && !isControlStatement(line)) break;
    }
  }
  if (lineNB == -1) return globalIdentifier;
  let match = text[lineNB].match(re);
  // Deal with old syntax here
  return match[2] == "" ? match[1] : match[2];
}

export function isFunction(
  range: Range,
  document: TextDocument,
  lineLength: number
): boolean {
  let start = new Position(range.start.line, range.end.character);
  let end = new Position(range.end.line, lineLength + 1);
  let rangeAfter = new Range(start, end);
  let wordsAfter: string = document.getText(rangeAfter);
  return /^\s*\(/.test(wordsAfter);
}

export function isControlStatement(line: string): boolean {
  let toCheck: RegExp[] = [
    /\s*\bif\b/,
    /\s*\bfor\b/,
    /\s*\bwhile\b/,
    /\s*\bcase\b/,
    /\s*\bswitch\b/,
    /\s*\breturn\b/,
  ];
  for (let re of toCheck) {
    if (re.test(line)) {
      return true;
    }
  }
  return false;
}

export function getLastEnumStructName(
  position: Position,
  document: TextDocument
): string {
  let lineNB = position.line;
  let re = /^\s*(?:enum\s+struct\s+)(\w+)/;
  let text = document.getText().split("\n");
  let match;
  let line: string;
  const maxiter = 150;
  let iter = 0;
  while (lineNB > 0 && iter < maxiter) {
    line = text[lineNB];
    if (line.match(/^\}/)) {
      // We are not in an enum struct.
      // This is not ideal, but I don't see a better way to do this for now.
      return globalIdentifier;
    }
    match = line.match(re);
    if (match) {
      break;
    }
    lineNB--;
    iter++;
  }
  if (lineNB == 0) return globalIdentifier;
  if (match !== null) {
    return match[1];
  }
  return globalIdentifier;
}
