import {
  CallHierarchyIncomingCall,
  CallHierarchyItem,
  CallHierarchyOutgoingCall,
  CancellationToken,
  CompletionItemKind,
  Position,
  Range,
  SymbolKind,
  TextDocument,
} from "vscode";
import { URI } from "vscode-uri";
import { FunctionItem } from "../Backend/Items/spFunctionItem";
import { MethodItem } from "../Backend/Items/spMethodItem";
import { getItemFromPosition } from "../Backend/spItemsGetters";
import { ItemsRepository } from "../Backend/spItemsRepository";
import { Providers } from "../Backend/spProviders";
import { findMainPath } from "../spUtils";

const callKind = [
  CompletionItemKind.Function,
  CompletionItemKind.Method,
  CompletionItemKind.Constructor,
];

export function provideIncomingCalls(
  item: CallHierarchyItem,
  token: CancellationToken,
  itemsRepo: ItemsRepository
): CallHierarchyIncomingCall[] {
  const mainPath = findMainPath(item.uri);
  if (mainPath === undefined) {
    return undefined;
  }
  const allItems = itemsRepo.getAllItems(URI.file(mainPath));
  const incomingCalls: CallHierarchyIncomingCall[] = [];
  const spItem = allItems.find(
    (e) =>
      e.range !== undefined &&
      e.range.isEqual(item.selectionRange) &&
      e.filePath === item.uri.fsPath
  );

  if (spItem === undefined || !spItem.references) {
    return undefined;
  }

  let refs = [...spItem.references];
  for (const caller of allItems) {
    if (!callKind.includes(caller.kind)) {
      continue;
    }
    const func = caller as FunctionItem | MethodItem;
    const uri = URI.file(func.filePath);
    const callerItem = new CallHierarchyItem(
      convertToSymbolKind(func.kind),
      func.name,
      func.detail,
      uri,
      func.fullRange,
      func.range
    );
    const ranges: Range[] = [];
    refs = refs.filter((ref) => {
      if (
        func.fullRange.contains(ref.range) &&
        uri.toString() === ref.uri.toString()
      ) {
        ranges.push(ref.range);
        return false;
      }
      return true;
    });
    if (ranges.length === 0) {
      continue;
    }
    incomingCalls.push(new CallHierarchyIncomingCall(callerItem, ranges));
  }
  return incomingCalls;
}

export function provideOutgoingCalls(
  item: CallHierarchyItem,
  token: CancellationToken,
  itemsRepo: ItemsRepository
): CallHierarchyOutgoingCall[] {
  const mainPath = findMainPath(item.uri);
  if (mainPath === undefined) {
    return undefined;
  }
  const allItems = itemsRepo.getAllItems(URI.file(mainPath));
  const outgoingCalls: CallHierarchyOutgoingCall[] = [];
  const spItem = allItems.find(
    (e) =>
      e.range !== undefined &&
      e.range.isEqual(item.selectionRange) &&
      e.filePath === item.uri.fsPath
  );

  if (spItem === undefined || !spItem.references) {
    return undefined;
  }

  for (const calle of allItems) {
    if (
      !callKind.includes(calle.kind) ||
      (calle.references && calle.references.length === 0)
    ) {
      continue;
    }

    const func = calle as FunctionItem | MethodItem;
    const uri = URI.file(spItem.filePath);
    const calleItem = new CallHierarchyItem(
      convertToSymbolKind(func.kind),
      func.name,
      func.detail,
      URI.file(func.filePath),
      func.fullRange,
      func.range
    );
    const ranges = calle.references
      .filter(
        (ref) =>
          spItem.fullRange.contains(ref.range) &&
          uri.toString() === ref.uri.toString()
      )
      .map((ref) => ref.range);
    if (ranges.length === 0) {
      continue;
    }

    outgoingCalls.push(new CallHierarchyOutgoingCall(calleItem, ranges));
  }

  return outgoingCalls;
}

export function prepareCallHierarchy(
  this: Providers,
  document: TextDocument,
  position: Position,
  token: CancellationToken
): CallHierarchyItem | CallHierarchyItem[] {
  const items = getItemFromPosition(this.itemsRepository, document, position);
  if (items === undefined) {
    return undefined;
  }

  const calleKinds = [CompletionItemKind.Function, CompletionItemKind.Method];
  let res = items.map((e) => {
    if (!calleKinds.includes(e.kind)) {
      return undefined;
    }
    return new CallHierarchyItem(
      e.kind === CompletionItemKind.Function
        ? SymbolKind.Function
        : SymbolKind.Method,
      e.name,
      e.detail,
      URI.file(e.filePath),
      e.fullRange,
      e.range
    );
  });
  res = res.filter((e) => e !== undefined);
  if (res.length === 0) {
    return undefined;
  }
  return res;
}

/**
 * Convert some CompletionItemKinds to a SymbolKind.
 * @param  {CompletionItemKind} funcKind  The CompletionItemKind to convert.
 * @returns SymbolKind
 */
function convertToSymbolKind(funcKind: CompletionItemKind): SymbolKind {
  let kind: SymbolKind;
  switch (funcKind) {
    case CompletionItemKind.Function:
      kind = SymbolKind.Function;
      break;
    case CompletionItemKind.Method:
      kind = SymbolKind.Method;
      break;
    case CompletionItemKind.Constructor:
      kind = SymbolKind.Constructor;
  }
  return kind;
}
