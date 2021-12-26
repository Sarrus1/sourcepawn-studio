import { URI } from "vscode-uri";

import { SPItem } from "./spItems";
import { ItemsRepository } from "./spItemsRepository";
import { findMainPath } from "../spUtils";
import { FileItems } from "./spFilesRepository";

/**
 * Returns an array of all the items parsed from a file and its known includes.
 * @param  {ItemsRepository} itemsRepo      The itemsRepository object constructed in the activation event.
 * @param  {URI} uri                        The URI of the file we are getting the items for.
 * @returns SPItem
 */
export function getAllItems(itemsRepo: ItemsRepository, uri: URI): SPItem[] {
  const mainPath = findMainPath(uri);
  if (mainPath !== undefined) {
    uri = URI.file(mainPath);
  }

  let includes = new Set<string>([uri.toString()]);
  let fileItems = itemsRepo.fileItems.get(uri.toString());
  if (fileItems === undefined) {
    return [];
  }

  itemsRepo.getIncludedFiles(fileItems, includes);
  return [].concat.apply([], Array.from(includes).map(getFileItems, itemsRepo));
}

/**
 * Callback used by the map function in getAllItems. Gets all the items from a parsed file, without its includes.
 * @param  {string} uri    The URI of the file we should get the items for.
 * @returns SPItem
 */
function getFileItems(uri: string): SPItem[] {
  let items: FileItems = this.fileItems.get(uri);
  return items === undefined ? Array.from(items.values()) : [];
}
