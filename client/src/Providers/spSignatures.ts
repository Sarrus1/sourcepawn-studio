import { TextDocument, Position } from "vscode";

interface SignatureAttributes {
  functionName: string;
  parameterCount: number;
}

export function getSignatureAttributes(
  document: TextDocument,
  position: Position
): SignatureAttributes {
  let line = document.getText().split("\n")[position.line];

  let blankReturn = { functionName: undefined, parameterCount: 0 };

  if (line[position.character - 1] === ")") {
    // We've finished this call
    return blankReturn;
  }

  let functionName: string = "";
  let parameterCount: number = 0;

  // Get first parenthesis
  let i: number = position.character - 1;
  let parenthesisCount: number = 0;
  let char: string;
  while (parenthesisCount < 1) {
    char = line[i];
    if (i < 0) {
      return blankReturn;
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
  let match: RegExpMatchArray = croppedLine.match(/(\w+)\s*$/);
  if (match) {
    functionName = match[1];
    return { functionName, parameterCount };
  }
  return blankReturn;
}

// TODO: Handle escaped quotation marks
function isInAStringOrArray(line: string, position: number): boolean {
  let doubleQuoteCount: number = 0;
  let singleQuoteCount: number = 0;
  let bracketCount: number = 0;
  let char: string;
  while (position >= 0) {
    char = line[position];
    if (char === '"') {
      doubleQuoteCount++;
    } else if (char === "'") {
      singleQuoteCount++;
    } else if (char === "{") {
      bracketCount++;
    } else if (char === "}") {
      bracketCount--;
    }
    position--;
  }
  return (
    singleQuoteCount % 2 === 1 ||
    doubleQuoteCount % 2 === 1 ||
    bracketCount !== 0
  );
}
