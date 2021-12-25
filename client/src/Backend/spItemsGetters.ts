import {
  CompletionItem,
  workspace as Workspace,
  TextDocument,
  CompletionList,
  CompletionItemKind,
} from "vscode";
import { basename, dirname, resolve } from "path";
import { URI } from "vscode-uri";
import { getAllPossibleIncludeFolderPaths } from "./spFileHandlers";

/**
 * Generate a CompletionList object of the possible includes file that can fit the already typed #include statement.
 * @param  {Set<string>} knownIncs    Set of parsed include files (.sp and .inc).
 * @param  {TextDocument} document    The document being edited.
 * @param  {string} tempName          The string that has already been typed in the #include statement.
 * @returns CompletionList
 */
export function getIncludeFileCompletionList(
  knownIncs: Set<string>,
  document: TextDocument,
  tempName: string
): CompletionList {
  const isQuoteInclude: boolean = tempName.includes('"');
  const incURIs = getAllPossibleIncludeFolderPaths(document.uri).map((e) =>
    URI.file(e)
  );
  const prevPath = tempName.replace(/((?:[^\'\<\/]+\/)+)+/, "$1");

  let items: CompletionItem[] = [];

  Array.from(knownIncs).forEach((e) =>
    incURIs.find((incURI) => {
      const fileMatchRe = RegExp(
        `${incURI.toString()}\\/${prevPath}[^<>:;,?"*|/]+\\.(?:inc|sp)$`
      );
      if (fileMatchRe.test(e)) {
        const path = URI.parse(e).fsPath;
        items.push({
          label: basename(path),
          kind: CompletionItemKind.File,
          detail: path,
        });
        return true;
      }
    })
  );

  const availableIncFolderPaths = new Set<string>();
  knownIncs.forEach((e) => {
    incURIs.forEach((incURI) => {
      const folderMatchRe = RegExp(
        `${incURI.toString()}\\/${prevPath}(\\w[^*/><?\\|:]+)\\/`
      );
      const match = e.match(folderMatchRe);
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
