import {
  CompletionItem,
  TextDocument,
  Position,
  CompletionList,
  CompletionItemKind,
  commands,
  SignatureHelp,
  Location,
} from "vscode";
import { basename } from "path";
import { URI } from "vscode-uri";

import { getTypeOfVariable } from "../../Backend/spItemsPropertyGetters";
import {
  getLastFunc,
  getLastESOrMM,
} from "../../Providers/spDefinitionProvider";
import { SPItem } from "../../Backend/Items/spItems";
import { getAllPossibleIncludeFolderPaths } from "../../Backend/spFileHandlers";
import { ItemsRepository } from "../../Backend/spItemsRepository";
import { isMethodCall } from "../../Backend/spUtils";
import { getAllInheritances } from "../../Backend/spItemsPropertyGetters";
import { MethodMapItem } from "../../Backend/Items/spMethodmapItem";
import { EnumStructItem } from "../../Backend/Items/spEnumStructItem";
import { FunctionItem } from "../../Backend/Items/spFunctionItem";
import { MethodItem } from "../../Backend/Items/spMethodItem";

const MP = [CompletionItemKind.Method, CompletionItemKind.Property];
const MPV = [
  CompletionItemKind.Method,
  CompletionItemKind.Property,
  CompletionItemKind.Variable,
];

/**
 * Generate a CompletionList object of the possible includes file that can fit the already typed #include statement.
 * @param  {Set<string>} knownIncs    Set of parsed include files (.sp and .inc).
 * @param  {TextDocument} document    The document being edited.
 * @param  {string} tempName          The string that has already been typed in the #include statement.
 * @returns CompletionList
 */
export function getIncludeFileCompletionList(
  knownIncs: Map<string, boolean>,
  document: TextDocument,
  tempName: string,
  useAp: boolean
): CompletionList {
  const incURIs = getAllPossibleIncludeFolderPaths(document.uri).map((e) =>
    URI.file(e)
  );
  const prevPath = tempName.replace(/((?:[^\'\<\/]+\/)+)+/, "$1");

  const items: CompletionItem[] = [];

  Array.from(knownIncs.keys()).forEach((e) =>
    incURIs.find((incURI) => {
      const fileMatchRe = RegExp(
        `${incURI.toString()}\\/${prevPath}[^<>:;,?"*|/]+\\.(?:inc|sp)$`
      );
      if (fileMatchRe.test(e)) {
        const path = URI.parse(e).fsPath;
        items.push({
          label: basename(path, ".inc"),
          insertText: `${basename(path, ".inc")}${useAp ? '"' : ">"}`,
          kind: CompletionItemKind.File,
          detail: path,
        });
        return true;
      }
      return false;
    })
  );

  const availableIncFolderPaths = new Set<string>();
  knownIncs.forEach((v, k) => {
    incURIs.forEach((incURI) => {
      const folderMatchRe = RegExp(
        `${incURI.toString()}\\/${prevPath}(\\w[^*/><?\\|:]+)\\/`
      );
      const match = k.match(folderMatchRe);
      if (match) {
        availableIncFolderPaths.add(`${incURI.toString()}/${match[1]}`);
      }
    });
  });

  availableIncFolderPaths.forEach((e) => {
    const path = URI.parse(e).fsPath;
    items.push({
      label: basename(path),
      kind: CompletionItemKind.Folder,
      detail: path,
    });
  });

  return new CompletionList(items);
}

/**
 * Returns a CompletionList object of all the objects available at that position's scope.
 * @param  {ItemsRepository} itemsRepo    The itemsRepository object constructed in the activation event.
 * @param  {TextDocument} document        The document the completions are requested for.
 * @param  {Position} position            The position at which the completions are requested.
 * @returns CompletionList
 */
export async function getCompletionListFromPosition(
  itemsRepo: ItemsRepository,
  document: TextDocument,
  position: Position
): Promise<CompletionList> {
  const allItems: SPItem[] = itemsRepo.getAllItems(document.uri);
  if (allItems.length === 0) {
    return new CompletionList();
  }

  const line = document.lineAt(position.line).text;
  const location = new Location(document.uri, position);
  const isMethod = isMethodCall(line, position);
  const lastFunc = getLastFunc(position, document, allItems);
  const lastESOrMM = getLastESOrMM(position, document.uri.fsPath, allItems);

  const positionalArguments = await getPositionalArguments(
    document,
    position,
    allItems,
    line
  );

  if (positionalArguments !== undefined) {
    return positionalArguments;
  }

  if (!isMethod) {
    return getNonMethodItems(allItems, location, lastFunc, lastESOrMM);
  }

  const { variableType, words } = getTypeOfVariable(
    line,
    position,
    allItems,
    lastFunc,
    lastESOrMM
  );

  if (!variableType) {
    return new CompletionList();
  }
  const variableTypeItem = allItems.find(
    (e) =>
      [CompletionItemKind.Class, CompletionItemKind.Struct].includes(e.kind) &&
      e.name === variableType
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

  const isMethodMap =
    words.length === 1 &&
    undefined !==
      allItems.find(
        (e) => e.name === words[0] && e.kind === CompletionItemKind.Class
      );

  return getMethodItems(
    allItems,
    variableTypes,
    isMethodMap,
    lastFunc,
    new Location(document.uri, position)
  );
}

function getMethodItems(
  allItems: SPItem[],
  variableTypes: (MethodMapItem | EnumStructItem)[],
  isMethodMap: boolean,
  lastFunc: MethodItem | FunctionItem,
  loc: Location
): CompletionList {
  const items = new Set<CompletionItem | undefined>();

  allItems.forEach((item) => {
    if (
      MPV.includes(item.kind) &&
      variableTypes.includes(item.parent as EnumStructItem | MethodMapItem) &&
      // Don't include the constructor of the methodmap
      !variableTypes.includes(item as EnumStructItem | MethodMapItem) &&
      // Don't include static methods if we are not calling a method from its type.
      // This handles suggestions for 'Database.Connect()' for example.
      isMethodMap === /\bstatic\b[^\(]*\(/.test(item.detail as string)
    ) {
      try {
        items.add(
          item.toCompletionItem(
            lastFunc,
            item.parent as MethodMapItem | EnumStructItem,
            loc
          )
        );
      } catch (err) {
        console.error(err);
      }
    }
  });

  items.delete(undefined);
  return new CompletionList(
    Array.from(items).filter((e) => e !== undefined) as CompletionItem[]
  );
}

function getNonMethodItems(
  allItems: SPItem[],
  location: Location,
  lastFunc: FunctionItem | MethodItem,
  lastMMorES: MethodMapItem | EnumStructItem | undefined
): CompletionList {
  const items: CompletionItem[] = [];

  allItems.forEach((item) => {
    if (!MP.includes(item.kind)) {
      const compItem = item.toCompletionItem(lastFunc, lastMMorES, location);
      if (compItem !== undefined) {
        items.push(compItem);
      }
    }
  });
  return new CompletionList(items);
}

/**
 * Return a CompletionList object of all the positional arguments of a function, if appropriate.
 * Return undefined otherwise.
 * @param  {TextDocument} document  The document the completions are requested for.
 * @param  {Position} position  The position at which the completions are requested.
 * @param  {SPItem[]} allItems  All the SPItems of the document, including the includes.
 * @param  {string} line  The line at which the completions are requested at.
 * @returns CompletionList|undefined
 */
async function getPositionalArguments(
  document: TextDocument,
  position: Position,
  allItems: SPItem[],
  line: string
): Promise<CompletionList | undefined> {
  const signatureHelp = (await commands.executeCommand(
    "vscode.executeSignatureHelpProvider",
    document.uri,
    position
  )) as SignatureHelp;
  if (signatureHelp === undefined || signatureHelp.signatures.length === 0) {
    return undefined;
  }
  if (line[position.character - 1] !== ".") {
    return undefined;
  }
  const match = signatureHelp.signatures[0].label.match(/(\w+)\(/);
  if (!match) {
    return undefined;
  }
  const params = allItems.filter(
    (e) => e.kind === CompletionItemKind.Variable && e.parent.name === match[1]
  );
  const completions = new CompletionList();
  completions.items = params.map((e) =>
    e.toCompletionItem(undefined, undefined, undefined, true)
  );
  return completions;
}
