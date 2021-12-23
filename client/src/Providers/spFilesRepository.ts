import { CompletionItem, workspace as Workspace } from "vscode";
import { dirname, resolve } from "path";
import { existsSync } from "fs";
import { URI } from "vscode-uri";
import { SPItem, Include, ConstantItem, KeywordItem } from "./spItems";
import { defaultConstantItems, defaultKeywordsItems } from "./spDefaultItems";
import { ItemsRepository } from "./spItemsRepository";

export class FileItems {
  completions: Map<string, SPItem>;
  includes: Include[];
  uri: string;

  constructor(uri: string) {
    this.completions = new Map();
    // Add constants only in one map.
    if (uri.includes("sourcemod.inc")) {
      makeNewItemsMap(this.completions);
    }
    this.includes = [];
    this.uri = uri;
  }

  add(id: string, completion: SPItem) {
    this.completions.set(id, completion);
  }

  get(id: string): SPItem {
    return this.completions.get(id);
  }

  getCompletions(repo: ItemsRepository): SPItem[] {
    let completions = [];
    for (let completion of this.completions.values()) {
      completions.push(completion);
    }
    return completions;
  }

  toCompletionResolve(item: CompletionItem): CompletionItem {
    item.label = item.label;
    item.documentation = item.documentation;
    return item;
  }

  addInclude(include: string, IsBuiltIn: boolean) {
    this.includes.push(new Include(include, IsBuiltIn));
  }

  resolve_import(
    file: string,
    documents: Set<string>,
    filePath: string,
    IsBuiltIn: boolean = false
  ) {
    let directoryPath = dirname(filePath);
    let inc_file: string;
    // If no extension is provided, it's a .inc file
    if (!/.sp\s*$/g.test(file) && !/.inc\s*$/g.test(file)) {
      file += ".inc";
    }
    let uri: string;
    let incFilePath = resolve(directoryPath, file);
    if (!existsSync(incFilePath)) {
      incFilePath = resolve(directoryPath, "include", file);
    }
    for (let parsedUri of documents.values()) {
      if (parsedUri == URI.file(incFilePath).toString()) {
        uri = parsedUri;
        break;
      }
    }

    if (uri === undefined) {
      let includes_dirs: string[] = Workspace.getConfiguration(
        "sourcepawn"
      ).get("optionalIncludeDirsPaths");
      for (let includes_dir of includes_dirs) {
        //inc_file = resolve(includes_dir, file);
        inc_file = resolve(
          Workspace.workspaceFolders.map((folder) => folder.uri.fsPath) +
            includes_dir +
            file
        );
        if (existsSync(inc_file)) {
          this.addInclude(URI.file(inc_file).toString(), IsBuiltIn);
          return;
        }
      }
    } else {
      this.addInclude(uri, IsBuiltIn);
    }
  }
}

function makeNewItemsMap(itemsMap): Map<string, SPItem> {
  for (let name of defaultConstantItems) {
    itemsMap.set(name, new ConstantItem(name));
  }
  for (let name of defaultKeywordsItems) {
    itemsMap.set(name, new KeywordItem(name));
  }
  return itemsMap;
}
