import { workspace as Workspace } from "vscode";
import { dirname, resolve } from "path";
import { existsSync } from "fs";
import { URI } from "vscode-uri";
import { SPItem, Include, ConstantItem, KeywordItem } from "./spItems";
import {
  defaultConstantItems,
  defaultKeywordsItems,
} from "../Providers/spDefaultItems";

export class FileItems extends Map {
  includes: Include[];
  uri: string;

  constructor(uri: string) {
    super();
    // Add constants only in one map.
    if (uri.includes("sourcemod.inc")) {
      defaultConstantItems.forEach((e) => this.set(e, new ConstantItem(e)));
      defaultKeywordsItems.forEach((e) => this.set(e, new KeywordItem(e)));
    }
    this.includes = [];
    this.uri = uri;
  }

  addInclude(include: string, IsBuiltIn: boolean) {
    this.includes.push(new Include(include, IsBuiltIn));
  }

  resolveImport(
    file: string,
    documents: Set<string>,
    filePath: string,
    IsBuiltIn: boolean = false
  ) {
    let directoryPath = dirname(filePath);
    let includeFile: string;
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
      let includeDirs: string[] = Workspace.getConfiguration("sourcepawn").get(
        "optionalIncludeDirsPaths"
      );
      for (let includeDir of includeDirs) {
        includeFile = resolve(
          Workspace.workspaceFolders.map((folder) => folder.uri.fsPath) +
            includeDir +
            file
        );
        if (existsSync(includeFile)) {
          this.addInclude(URI.file(includeFile).toString(), IsBuiltIn);
          return;
        }
      }
    } else {
      this.addInclude(uri, IsBuiltIn);
    }
  }
}
