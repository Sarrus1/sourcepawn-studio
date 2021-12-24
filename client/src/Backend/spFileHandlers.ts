import { TextDocumentChangeEvent, FileCreateEvent } from "vscode";
import { URI } from "vscode-uri";
import { extname } from "path";
import { ItemsRepository } from "./spItemsRepository";
import { FileItems } from "./spFilesRepository";
import { parseText, parseFile } from "../Parser/spParser";

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
export function handleDocumentChange(
  itemsRepo: ItemsRepository,
  event: TextDocumentChangeEvent
): void {
  // if (event.contentChanges.length > 0) {
  //   let textChange = event.contentChanges[0].text;
  //   // Don't parse the document every character change.
  //   if (/\w+/.test(textChange)) {
  //     return;
  //   }
  // }
  const fileUri = event.document.uri.toString();
  const filePath: string = event.document.uri.fsPath.replace(".git", "");

  let fileItems = new FileItems(fileUri);
  itemsRepo.documents.add(fileUri);
  // We use parseText here, otherwise, if the user didn't save the file, the changes wouldn't be registered.
  try {
    parseText(event.document.getText(), filePath, fileItems, itemsRepo);
  } catch (error) {
    console.log(error);
  }
  itemsRepo.readUnscannedImports(fileItems.includes);
  itemsRepo.fileItems.set(fileUri, fileItems);
}

/**
 * Generic callback for newly added/created documents. Parses the file.
 * @param  {ItemsRepository} itemsRepo    The itemsRepository object constructed in the activation event.
 * @param  {URI} uri                      The URI of the document.
 * @returns void
 */
export function newDocumentCallback(
  itemsRepo: ItemsRepository,
  uri: URI
): void {
  const filePath: string = uri.fsPath;

  if (
    ![".inc", ".sp"].includes(extname(uri.fsPath)) ||
    filePath.includes(".git")
  ) {
    return;
  }

  let fileItems: FileItems = new FileItems(uri.toString());
  itemsRepo.documents.add(uri.toString());
  try {
    parseFile(filePath, fileItems, itemsRepo);
  } catch (error) {
    console.error(error);
  }
  itemsRepo.readUnscannedImports(fileItems.includes);
  itemsRepo.fileItems.set(uri.toString(), fileItems);
}
