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
  let quoteCount: number = 0;
  let char: string = "";
  while (parenthesisCount < 1) {
    char = line[i];
    if (i < 0) {
      return blankReturn;
    }
    if (char === "(") {
      parenthesisCount++;
    } else if (char === ")") {
      parenthesisCount--;
    } else if (char === ",") {
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
