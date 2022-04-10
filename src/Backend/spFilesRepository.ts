import { workspace as Workspace } from "vscode";
import { dirname, resolve } from "path";
import { existsSync } from "fs";
import { URI } from "vscode-uri";

import { Include, SPItem } from "./Items/spItems";
import { KeywordItem } from "./Items/spKeywordItem";
import { ConstantItem } from "./Items/spConstantItem";
import {
  defaultConstantItems,
  defaultKeywordsItems,
} from "../Providers/spDefaultItems";
import { getIncludeExtension } from "./spUtils";
import { ParsedID } from "../Parser/interfaces";

export class FileItems extends Map<string, SPItem> {
  includes: Include[];
  uri: string;
  tokens: ParsedID[];

  constructor(uri: string) {
    super();
    // Add constants only in one map.
    if (uri.includes("sourcemod.inc")) {
      defaultConstantItems.forEach((e) => this.set(e, new ConstantItem(e)));
      defaultKeywordsItems.forEach((e) => this.set(e, new KeywordItem(e)));
    }
    this.includes = [];
    this.uri = uri;
    this.tokens = [];
  }

  /**
   * Add a new Include to the array of parsed includes for this file.
   * @param  {string} uri          URI of the parsed include.
   * @param  {boolean} IsBuiltIn   Whether or not the parsed include is a Sourcemod builtin.
   * @returns void
   */
  addInclude(uri: string, IsBuiltIn: boolean): void {
    this.includes.push(new Include(uri, IsBuiltIn));
  }

  /**
   * Resolve an include from its #include directive and the file it was imported in.
   * @param  {string} includeText       The text inside the #include directive.
   * @param  {Set<string>} documents    The documents (.inc/.sp) that have been found in the SMHome folder,
   *                                    include folder, optionalIncludes folder, etc.
   * @param  {string} filePath          The path of the file the include was imported in.
   * @param  {boolean=false} IsBuiltIn  Whether or not the parsed file is a Sourcemod builtin.
   * @returns void
   */
  resolveImport(
    includeText: string,
    documents: Set<string>,
    filePath: string,
    IsBuiltIn: boolean = false
  ): void {
    const SMHome: string = Workspace.getConfiguration(
      "sourcepawn",
      Workspace.getWorkspaceFolder(URI.file(filePath))
    ).get("SourcemodHome");
    const directoryPath = dirname(filePath);
    includeText = getIncludeExtension(includeText);
    let incFilePath = resolve(directoryPath, includeText);
    if (!existsSync(incFilePath)) {
      incFilePath = resolve(directoryPath, "include", includeText);
      if (!existsSync(incFilePath)) {
        incFilePath = resolve(SMHome, includeText);
      }
    }
    for (let parsedUri of documents.values()) {
      if (parsedUri == URI.file(incFilePath).toString()) {
        this.addInclude(parsedUri, IsBuiltIn);
        return;
      }
    }

    let includeDirs: string[] = Workspace.getConfiguration("sourcepawn").get(
      "optionalIncludeDirsPaths"
    );
    for (let includeDir of includeDirs) {
      let includeFile = resolve(
        ...Workspace.workspaceFolders
          .map((folder) => folder.uri.fsPath)
          .concat(includeDir, includeText)
      );
      if (existsSync(includeFile)) {
        this.addInclude(URI.file(includeFile).toString(), IsBuiltIn);
        return;
      }
    }
  }
}
