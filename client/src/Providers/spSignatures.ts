import { TextDocument, Position } from "vscode";

interface SignatureAttributes {
  croppedLine: string;
  parameterCount: number;
}

export function getSignatureAttributes(
  document: TextDocument,
  position: Position
): SignatureAttributes {
  let lineNB: number = position.line;
  let lines = document.getText().split("\n");
  let line = lines[lineNB];

  let blankReturn = { croppedLine: undefined, parameterCount: 0 };

  if (line[position.character - 1] === ")") {
    // We've finished this call
    return blankReturn;
  }

  let parameterCount: number = 0;

  let i: number = position.character - 1;
  let parenthesisCount: number = 0;
  let char: string;
  while (parenthesisCount < 1) {
    char = line[i];
    if (i < 0) {
      // If we didn't find an opening parenthesis, go to the preceding line
      // if it exists.
      if (lineNB >= 0) {
        lineNB--;
        line = lines[lineNB];
        i = line.length;
        continue;
      } else {
        return blankReturn;
      }
    }
    if (char === "(") {
      parenthesisCount++;
    } else if (char === ")") {
      parenthesisCount--;
    } else if (char === "," && !isInAStringOrArray(line, i)) {
      parameterCount++;
    }
    i--;
  }
  let croppedLine: string = line.slice(0, i + 1);
  return { croppedLine, parameterCount };
  /*
  let match: RegExpMatchArray = croppedLine.match(/(\w+)\s*$/);
  if (match) {
    functionName = match[1];
    return { functionName, parameterCount };
  }
  return blankReturn;
  */
}

function isInAStringOrArray(line: string, position: number): boolean {
  let doubleQuoteCount: number = 0;
  let foundDoubleQuote: boolean = false;
  let singleQuoteCount: number = 0;
  let foundSingleQuote: boolean = false;
  let bracketCount: number = 0;
  let char: string;

  while (position >= 0) {
    char = line[position];
    if (char === '"') {
      foundDoubleQuote = true;
      doubleQuoteCount++;
    } else if (char === "'") {
      foundSingleQuote = true;
      singleQuoteCount++;
    } else if (char === "{") {
      bracketCount++;
    } else if (char === "}") {
      bracketCount--;
    } else if (char === "\\") {
      if (foundDoubleQuote) {
        foundDoubleQuote = false;
        doubleQuoteCount--;
      } else if (foundSingleQuote) {
        foundSingleQuote = false;
        singleQuoteCount--;
      }
    }
    position--;
  }
  return (
    singleQuoteCount % 2 === 1 ||
    doubleQuoteCount % 2 === 1 ||
    bracketCount !== 0
  );
}
