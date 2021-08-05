import * as vscode from "vscode";

export function GetLastFuncName(
  lineNB: number,
  document: vscode.TextDocument
): string {
  let re = /(?:static|native|stock|public|forward)?\s*(?:[a-zA-Z\-_0-9]:)?([^\s]+)\s*([A-Za-z_]*)\s*\(([^\)]*)(?:\)?)(?:\s*)(?:\{?)(?:\s*)(?:[^\;\s]*);?\s*$/;
  let text = document.getText().split("\n");
  let Match;
  let line: string;
  for (lineNB; lineNB > 0; lineNB--) {
    line = text[lineNB];
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
  if (lineNB == 0) return undefined;
  let match = text[lineNB].match(re);
  // Deal with old syntax here
  return match[2] == "" ? match[1] : match[2];
}

export function isFunction(
	range: vscode.Range,
	document: vscode.TextDocument,
	lineLength: number
): boolean {
	let start = new vscode.Position(range.start.line, range.end.character);
	let end = new vscode.Position(range.end.line, lineLength + 1);
	let rangeAfter = new vscode.Range(start, end);
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
		/\s*\breturn\b/
  ];
  for (let re of toCheck) {
    if (re.test(line)) {
      return true;
    }
  }
  return false;
}
