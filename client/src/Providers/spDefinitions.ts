import { TextDocument, Range, Position, CompletionItemKind } from "vscode";
import { globalIdentifier } from "./spGlobalIdentifier";
import { SPItem } from "./spItems";

export function GetLastFuncName(
  position: Position,
  document: TextDocument,
  allItems: SPItem[]
): string {
  let func = allItems.find(
    (e) =>
      [CompletionItemKind.Function, CompletionItemKind.Method].includes(
        e.kind
      ) &&
      e.file === document.uri.fsPath &&
      e.fullRange != undefined &&
      e.fullRange.contains(position)
  );
  return func != undefined ? func.name : globalIdentifier;
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

export function getLastEnumStructNameOrMethodMap(
  position: Position,
  document: TextDocument,
  allItems: SPItem[]
): string {
  let enumStruct = allItems.find(
    (e) =>
      [CompletionItemKind.Struct, CompletionItemKind.Class].includes(e.kind) &&
      e.file === document.uri.fsPath &&
      e.fullRange != undefined &&
      e.fullRange.contains(position)
  );
  return enumStruct != undefined ? enumStruct.name : globalIdentifier;
}
