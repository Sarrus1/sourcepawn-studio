import {
  TextDocumentChangeEvent,
  FileCreateEvent,
  workspace as Workspace,
} from "vscode";
import { URI } from "vscode-uri";
import { resolve, dirname, join, extname } from "path";
import { existsSync } from "fs";

import { ItemsRepository } from "./spItemsRepository";
import { FileItem } from "./spFilesRepository";
import { parseText, parseFile } from "../Parser/spParser";
import { getAllMethodmaps } from "./spItemsGetters";
import { PreProcessor } from "../Parser/PreProcessor/spPreprocessor";
import { Include } from "./Items/spItems";

/**
 * Handle the addition of a document by forwarding it to the newDocumentCallback function.
 * @param  {ItemsRepository} itemsRepo      The itemsRepository object constructed in the activation event.
 * @param  {FileCreateEvent} event          The file created event triggered by the creation of the file.
 * @returns void
 */
export function handleAddedDocument(
  itemsRepo: ItemsRepository,
  event: FileCreateEvent
): void {
  event.files.forEach((e) => newDocumentCallback(itemsRepo, e));
}

/**
 * Handle the changes in a document by creating a new FileItem object and parsing the file, even if it wasn't saved.
 * @param  {ItemsRepository} itemsRepo      The itemsRepository object constructed in the activation event.
 * @param  {TextDocumentChangeEvent} event  The document change event triggered by the file change.
 * @returns void
 */
export async function documentChangeCallback(
  itemsRepo: ItemsRepository,
  event: TextDocumentChangeEvent
) {
  const fileUri = event.document.uri.toString();
  const filePath = event.document.uri.fsPath.replace(".git", "");

  // Hack to make the function non blocking, and not prevent the completionProvider from running.
  await new Promise((resolve) => setTimeout(resolve, 75));

  const fileItem = new FileItem(fileUri);

  let text = event.document.getText();

  const preprocessor = new PreProcessor(text.split("\n"), fileItem, itemsRepo);
  fileItem.text = preprocessor.preProcess();
  text = fileItem.text;
  itemsRepo.documents.set(event.document.uri.toString(), false);
  // We use parseText here, otherwise, if the user didn't save the file, the changes wouldn't be registered.
  try {
    parseText(text, filePath, fileItem, itemsRepo, false);
  } catch (error) {
    console.log(error);
  }
  itemsRepo.fileItems.set(fileUri, fileItem);

  readUnscannedImports(itemsRepo, fileItem.includes);

  resolveMethodmapInherits(itemsRepo, event.document.uri);

  parseText(text, filePath, fileItem, itemsRepo, true);
}

/**
 * Generic callback for newly added/created documents. Parses the file.
 * @param  {ItemsRepository} itemsRepo    The itemsRepository object constructed in the activation event.
 * @param  {URI} uri                      The URI of the document.
 * @returns void
 */
export async function newDocumentCallback(
  itemsRepo: ItemsRepository,
  uri: URI
) {
  const filePath = uri.fsPath;

  if (itemsRepo.fileItems.has(uri.toString())) {
    // Don't parse the document again if it was already.
    return;
  }

  if (!isSPFile(uri.fsPath)) {
    return;
  }

  const fileItem: FileItem = new FileItem(uri.toString());
  itemsRepo.documents.set(uri.toString(), false);

  try {
    parseFile(filePath, fileItem, itemsRepo, false);
  } catch (error) {
    console.error(error);
  }
  itemsRepo.fileItems.set(uri.toString(), fileItem);

  readUnscannedImports(itemsRepo, fileItem.includes);

  resolveMethodmapInherits(itemsRepo, uri);

  // Parse token references.
  parseFile(filePath, fileItem, itemsRepo, true);
  itemsRepo.fileItems.forEach((fileItems, k) => {
    fileItems.includes.forEach((e) => {
      const uri = URI.parse(e.uri);
      if (itemsRepo.documents.get(uri.toString())) {
        return;
      }
      parseFile(
        uri.fsPath,
        itemsRepo.fileItems.get(uri.toString()),
        itemsRepo,
        true
      );
      itemsRepo.documents.set(uri.toString(), true);
    });
  });
}

/**
 * Return all the possible include directories paths, such as SMHome, etc. The function will only return existing paths.
 * @param  {URI} uri                          The URI of the file from which we are trying to read the include.
 * @param  {boolean} onlyOptionalPaths        Whether or not the function only return the optionalIncludeFolderPaths.
 * @returns string
 */
export function getAllPossibleIncludeFolderPaths(
  uri: URI,
  onlyOptionalPaths = false
): string[] {
  let possibleIncludePaths: string[] = [];
  const workspaceFolder = Workspace.getWorkspaceFolder(uri);

  possibleIncludePaths = Workspace.getConfiguration(
    "sourcepawn",
    workspaceFolder
  ).get("optionalIncludeDirsPaths");
  possibleIncludePaths = possibleIncludePaths.map((e) =>
    resolve(workspaceFolder === undefined ? "" : workspaceFolder.uri.fsPath, e)
  );

  if (onlyOptionalPaths) {
    return possibleIncludePaths;
  }

  const smHome = Workspace.getConfiguration(
    "sourcepawn",
    workspaceFolder
  ).get<string>("SourcemodHome");

  if (smHome !== undefined) {
    possibleIncludePaths.push(smHome);
  }

  const scriptingFolder = dirname(uri.fsPath);
  possibleIncludePaths.push(scriptingFolder);
  possibleIncludePaths.push(join(scriptingFolder, "include"));

  return possibleIncludePaths.filter((e) => e !== "" && existsSync(e));
}

/**
 * Deal with all the tmpParents properties of methodmaps items post parsing.
 * @param  {ItemsRepository} itemsRepo The itemsRepository object constructed in the activation event.
 * @param  {URI} uri  The uri of the document to check the methodmaps for (will check the includes as well).
 * @returns void
 */
function resolveMethodmapInherits(itemsRepo: ItemsRepository, uri: URI): void {
  const methodmaps = getAllMethodmaps(itemsRepo, uri);
  methodmaps.forEach((v, k) => {
    if (v.tmpParent === undefined) {
      return;
    }
    const parent = methodmaps.get(v.tmpParent);
    if (parent === undefined) {
      return;
    }
    v.parent = parent;
    v.tmpParent = undefined;
  });
}

/**
 * Returns whether or not a file is a .sp or .inc files and filters out .git files.
 * @param  {string} filePath  Path of the file to check.
 * @returns boolean
 */
export function isSPFile(filePath: string): boolean {
  const fileExt = extname(filePath);
  return [".sp", ".inc"].includes(fileExt) && !filePath.includes(".git");
}

/**
 * Recursively read the unparsed includes from a array of Include objects.
 * @param  {ItemsRepository} itemsRepo    The itemsRepository object constructed in the activation event.
 * @param  {Include[]} includes           The array of Include objects to parse.
 * @returns void
 */
function readUnscannedImports(
  itemsRepo: ItemsRepository,
  includes: Map<string, Include>
): void {
  const debug = ["messages", "verbose"].includes(
    Workspace.getConfiguration("sourcepawn").get("trace.server")
  );
  includes.forEach((include) => {
    if (debug) console.log("reading", include.uri.toString());

    const filePath = URI.parse(include.uri).fsPath;

    const fileItem = itemsRepo.fileItems.get(include.uri);
    if (fileItem) {
      if (fileItem.uri.endsWith("sourcemod.inc")) {
        // Sourcemod.inc has some hardcoded items, we account for them here.
        if (fileItem.items.length > 33) {
          return;
        }
      } else if (fileItem.items.length > 0) {
        return;
      }
    }

    if (!existsSync(filePath)) {
      return;
    }

    if (debug) console.log("found", include.uri.toString());

    let fileItems: FileItem = new FileItem(include.uri);
    try {
      parseFile(filePath, fileItems, itemsRepo, false);
    } catch (err) {
      console.error(err, include.uri.toString());
    }
    if (debug) console.log("parsed", include.uri.toString());

    itemsRepo.fileItems.set(include.uri, fileItems);
    if (debug) console.log("added", include.uri.toString());

    readUnscannedImports(itemsRepo, fileItems.includes);
  });
}
