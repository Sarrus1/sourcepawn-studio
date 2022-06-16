import {
  TextDocument,
  Position,
  CancellationToken,
  CompletionItemKind,
} from "vscode";

import { getTypeOfVariable } from "../Backend/spItemsPropertyGetters";
import { ItemsRepository } from "../Backend/spItemsRepository";
import { getLastFunc, getLastESOrMM } from "./spDefinitionProvider";
import { getAllInheritances } from "../Backend/spItemsPropertyGetters";
import { MethodMapItem } from "../Backend/Items/spMethodmapItem";
import { EnumStructItem } from "../Backend/Items/spEnumStructItem";

interface SignatureAttributes {
  croppedLine: string;
  parameterCount: number;
}

export function getSignatureAttributes(
  document: TextDocument,
  position: Position
): SignatureAttributes {
  let lineNB: number = position.line;
  const lines = document.getText().split("\n");
  let line = lines[lineNB];

  const blankReturn = { croppedLine: undefined, parameterCount: 0 };

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
  const croppedLine: string = line.slice(0, i + 1);
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
  const blankReturn = {
    signatures: [],
    activeSignature: 0,
    activeParameter: 0,
  };
  const { croppedLine, parameterCount } = getSignatureAttributes(
    document,
    position
  );
  if (croppedLine === undefined) {
    return blankReturn;
  }

  // Check if it's a method
  let match = croppedLine.match(/\.(\w+)$/);
  if (match) {
    const methodName = match[1];
    const allItems = itemsRepo.getAllItems(document.uri);
    const lastFunc = getLastFunc(position, document, allItems);
    const newPos = new Position(1, croppedLine.length);
    const lastEnumStructOrMethodMap = getLastESOrMM(
      position,
      document.uri.fsPath,
      allItems
    );
    const { variableType, words } = getTypeOfVariable(
      croppedLine,
      newPos,
      allItems,
      lastFunc,
      lastEnumStructOrMethodMap
    );
    const variableTypeItem = allItems.find(
      (e) =>
        [CompletionItemKind.Class, CompletionItemKind.Struct].includes(
          e.kind
        ) && e.name === variableType
    ) as MethodMapItem | EnumStructItem;

    let variableTypes: (MethodMapItem | EnumStructItem)[];
    if (variableTypeItem.kind === CompletionItemKind.Class) {
      variableTypes = getAllInheritances(
        variableTypeItem as MethodMapItem,
        allItems
      );
    } else {
      variableTypes = [variableTypeItem as EnumStructItem];
    }

    const items = itemsRepo
      .getAllItems(document.uri)
      .filter(
        (item) =>
          (item.kind === CompletionItemKind.Method ||
            item.kind === CompletionItemKind.Property) &&
          variableTypes.includes(item.parent as MethodMapItem) &&
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
    const methodMapName = match[1];
    const items = itemsRepo
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
