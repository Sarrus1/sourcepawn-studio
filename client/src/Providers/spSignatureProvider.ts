import {
  TextDocument,
  Position,
  CancellationToken,
  CompletionItemKind,
} from "vscode";
import { ItemsRepository } from "../Backend/spItemsRepository";
import {
  getLastFuncName,
  getLastEnumStructNameOrMethodMap,
} from "./spDefinitionProvider";

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
        if (line == undefined) {
          return blankReturn;
        }
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

export function signatureProvider(
  itemsRepo: ItemsRepository,
  document: TextDocument,
  position: Position,
  token: CancellationToken
) {
  let blankReturn = {
    signatures: [],
    activeSignature: 0,
    activeParameter: 0,
  };
  let { croppedLine, parameterCount } = getSignatureAttributes(
    document,
    position
  );
  if (croppedLine === undefined) {
    return blankReturn;
  }
  // Check if it's a method
  let match = croppedLine.match(/\.(\w+)$/);
  if (match) {
    let methodName = match[1];
    let allItems = itemsRepo.getAllItems(document.uri);
    let lastFuncName = getLastFuncName(position, document, allItems);
    let newPos = new Position(1, croppedLine.length);
    let {
      lastEnumStructOrMethodMap,
      isAMethodMap,
    } = getLastEnumStructNameOrMethodMap(position, document, allItems);
    let { variableType, words } = itemsRepo.getTypeOfVariable(
      croppedLine,
      newPos,
      allItems,
      lastFuncName,
      lastEnumStructOrMethodMap
    );
    let variableTypes: string[] = itemsRepo.getAllInheritances(
      variableType,
      allItems
    );
    let items = itemsRepo
      .getAllItems(document.uri)
      .filter(
        (item) =>
          (item.kind === CompletionItemKind.Method ||
            item.kind === CompletionItemKind.Property) &&
          variableTypes.includes(item.parent) &&
          item.name === methodName
      );
    return {
      signatures: items.map((e) => e.toSignature()),
      activeParameter: parameterCount,
      activeSignature: 0,
    };
  }
  // Match for new keywords
  match = croppedLine.match(/new\s+(\w+)/);
  if (match) {
    let methodMapName = match[1];
    let items = itemsRepo
      .getAllItems(document.uri)
      .filter(
        (item) =>
          item.kind === CompletionItemKind.Constructor &&
          item.name === methodMapName
      );
    return {
      signatures: items.map((e) => e.toSignature()),
      activeParameter: parameterCount,
      activeSignature: 0,
    };
  }

  match = croppedLine.match(/(\w+)$/);
  if (!match) {
    return blankReturn;
  }
  if (["if", "for", "while", "case", "switch", "return"].includes(match[1])) {
    return blankReturn;
  }
  let items = itemsRepo
    .getAllItems(document.uri)
    .filter(
      (item) =>
        item.name === match[1] &&
        [CompletionItemKind.Function, CompletionItemKind.Interface].includes(
          item.kind
        )
    );
  if (items === undefined) {
    return blankReturn;
  }
  // Sort by size of description
  items = items.sort((a, b) => b.description.length - a.description.length);
  return {
    signatures: items.map((e) => e.toSignature()),
    activeParameter: parameterCount,
    activeSignature: 0,
  };
}
