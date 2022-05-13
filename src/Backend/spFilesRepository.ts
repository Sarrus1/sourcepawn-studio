import { Range, workspace as Workspace } from "vscode";
import { dirname, resolve } from "path";
import { existsSync } from "fs";
import { URI } from "vscode-uri";

import { Include, SPItem } from "./Items/spItems";
import { KeywordItem } from "./Items/spKeywordItem";
import { ConstantItem } from "./Items/spConstantItem";
import {
  defaultConstantItems,
  defaultKeywordsItems,
  hardcodedDefines,
} from "../Providers/spDefaultItems";
import { getIncludeExtension } from "./spUtils";
import { ParsedID } from "../Parser/interfaces";
import { MethodMapItem } from "./Items/spMethodmapItem";
import { spParserArgs } from "../Parser/interfaces";
import { parsedLocToRange } from "../Parser/utils";
import { reservedTokens } from "../Misc/spConstants";
import { TypedefItem } from "./Items/spTypedefItem";
import { DefineItem } from "./Items/spDefineItem";

export interface parsedToken {
  id: string;
  range: Range;
}

export class FileItem {
  includes: Map<string, Include>;
  uri: string;
  tokens: parsedToken[];
  methodmaps: Map<string, MethodMapItem>;
  items: SPItem[];

  constructor(uri: string) {
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
            true,
            zeroRange
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
    this.tokens = [];
    this.methodmaps = new Map<string, MethodMapItem>();
  }

  /**
   * Add a new Include to the array of parsed includes for this file.
   * @param  {string} uri          URI of the parsed include.
   * @param  {boolean} IsBuiltIn   Whether or not the parsed include is a Sourcemod builtin.
   * @returns void
   */
  addInclude(uri: string, range: Range, IsBuiltIn: boolean): void {
    this.includes.set(uri, new Include(uri, range, IsBuiltIn));
  }

  /**
   * Resolve an include from its #include directive and the file it was imported in.
   * @param  {string} includeText       The text inside the #include directive.
   * @param  {Set<string>} documents    The documents (.inc/.sp) that have been found in the SMHome folder,
   *                                    include folder, optionalIncludes folder, etc.
   * @param  {string} filePath          The path of the file the include was imported in.
   * @param  {boolean} IsBuiltIn        Whether or not the parsed file is a Sourcemod builtin.
   * @returns void
   */
  resolveImport(
    includeText: string,
    documents: Map<string, boolean>,
    filePath: string,
    range: Range,
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

    const uri = URI.file(incFilePath);
    if (documents.has(uri.toString())) {
      this.addInclude(uri.toString(), range, IsBuiltIn);
      return;
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
        this.addInclude(URI.file(includeFile).toString(), range, IsBuiltIn);
        return;
      }
    }
  }

  /**
   * Add a parsed token to the array of parsed token by taking into account the offset of the error
   * recovery.
   * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
   * @param  {ParsedID} id  The parsed ID of the token.
   * @returns void
   */
  pushToken(parserArgs: spParserArgs, id: ParsedID): void {
    if (reservedTokens.has(id.id)) {
      return;
    }
    const range = parsedLocToRange(id.loc, parserArgs);

    // Prevent duplicates in the tokens array.
    const length = this.tokens.length;
    if (length > 0 && this.tokens[length - 1].range.isEqual(range)) {
      return;
    }
    this.tokens.push({ id: id.id, range });
  }
}
