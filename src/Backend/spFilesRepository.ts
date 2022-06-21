import { Range, workspace as Workspace } from "vscode";
import { dirname, resolve } from "path";
import { existsSync } from "fs";
import { URI } from "vscode-uri";
import { QueryCapture } from "web-tree-sitter";

import { Include, SPItem } from "./Items/spItems";
import { KeywordItem } from "./Items/spKeywordItem";
import { ConstantItem } from "./Items/spConstantItem";
import {
  defaultConstantItems,
  defaultKeywordsItems,
  hardcodedDefines,
} from "../Providers/spDefaultItems";
import { getIncludeExtension } from "./spUtils";
import { MethodMapItem } from "./Items/spMethodmapItem";
import { TypedefItem } from "./Items/spTypedefItem";
import { DefineItem } from "./Items/spDefineItem";

export interface parsedToken {
  id: string;
  range: Range;
}

export class FileItem {
  includes: Map<string, Include>;
  uri: string;
  methodmaps: Map<string, MethodMapItem>;
  items: SPItem[];

  symbols: QueryCapture[];
  /**
   * Preprocessed text
   */
  text: string;
  defines: Map<string, string>;
  failedParse: number;

  constructor(uri: string, failedParse = 0) {
    this.items = [];
    // Add constants only in one map.
    if (uri.includes("sourcemod.inc")) {
      defaultConstantItems.forEach((e) => this.items.push(new ConstantItem(e)));
      defaultKeywordsItems.forEach((e) => this.items.push(new KeywordItem(e)));
      const zeroRange = new Range(0, 0, 0, 0);
      hardcodedDefines.forEach((e) =>
        this.items.push(
          new DefineItem(
            e,
            "",
            "Hardcoded constant",
            URI.parse(uri).fsPath,
            zeroRange,
            zeroRange,
            undefined
          )
        )
      );

      this.items.push(
        new TypedefItem(
          "Function",
          "",
          URI.parse(uri).fsPath,
          "Hardcoded constant",
          "",
          zeroRange,
          zeroRange,
          []
        )
      );
    }
    this.includes = new Map();
    this.uri = uri;
    this.symbols = [];
    this.methodmaps = new Map<string, MethodMapItem>();
    this.defines = new Map();
    this.failedParse = failedParse;
  }

  /**
   * Add a new Include to the array of parsed includes for this file.
   * @param  {string} uri          URI of the parsed include.
   * @param  {Range} range         Range of the parsed include.
   * @returns void
   */
  addInclude(uri: string, range: Range): void {
    this.includes.set(uri, new Include(uri, range));
  }

  /**
   * Resolve an include from its #include directive and the file it was imported in.
   * @param  {string} includeText       The text inside the #include directive.
   * @param  {Set<string>} documents    The documents (.inc/.sp) that have been found in the SMHome folder,
   *                                    include folder, optionalIncludes folder, etc.
   * @param  {string} filePath          The path of the file the include was imported in.
   * @returns string
   */
  resolveImport(
    includeText: string,
    documents: Map<string, boolean>,
    filePath: string,
    range: Range
  ): string {
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

    const uri = URI.file(incFilePath);
    if (documents.has(uri.toString())) {
      this.addInclude(uri.toString(), range);
      return uri.toString();
    }

    const includeDirs: string[] = Workspace.getConfiguration("sourcepawn").get(
      "optionalIncludeDirsPaths"
    );
    for (const includeDir of includeDirs) {
      const includeFile = resolve(
        ...Workspace.workspaceFolders
          .map((folder) => folder.uri.fsPath)
          .concat(includeDir, includeText)
      );
      if (existsSync(includeFile)) {
        this.addInclude(URI.file(includeFile).toString(), range);
        return uri.toString();
      }
    }
    return undefined;
  }
}
