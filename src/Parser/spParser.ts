import { Diagnostic, DiagnosticSeverity, Range } from "vscode";
import { existsSync, readFileSync } from "fs";
import { resolve, dirname } from "path";
import { URI } from "vscode-uri";

import { ItemsRepository } from "../Backend/spItemsRepository";
import { FileItem } from "../Backend/spFilesRepository";
import { getNextScope, parsedLocToRange } from "./utils";
import { parserDiagnostics } from "../Providers/Linter/compilerDiagnostics";
import { spParserArgs } from "./interfaces";
import { Semantics } from "./Semantics/spSemantics";
import { PreProcessor } from "./PreProcessor/spPreprocessor";
import { parser } from "../spIndex";
import * as TreeSitter from "web-tree-sitter";
import { readVariable } from "./readVariableNew";

export function parseFile(
  file: string,
  fileItem: FileItem,
  itemsRepository: ItemsRepository,
  searchTokens: boolean,
  IsBuiltIn: boolean
) {
  if (!existsSync(file)) {
    return;
  }
  let data = readFileSync(file, "utf-8");

  // Test for symbolic links
  let match = data.match(/^(?:\.\.\/)+(?:[\/\w\-])+\.\w+/);
  if (match !== null) {
    let folderpath = dirname(file);
    file = resolve(folderpath, match[0]);
    data = readFileSync(file, "utf-8");
  }
  if (!searchTokens) {
    const preprocessor = new PreProcessor(
      data.split("\n"),
      fileItem,
      itemsRepository
    );
    data = preprocessor.preProcess();
    fileItem.text = data;
  }
  parseText(
    fileItem.text,
    file,
    fileItem,
    itemsRepository,
    searchTokens,
    IsBuiltIn
  );
}

export function parseText(
  data: string,
  file: string,
  fileItem: FileItem,
  itemsRepository: ItemsRepository,
  searchTokens: boolean,
  isBuiltIn: boolean,
  offset: number = 0,
  range?: Range
): boolean {
  if (data === undefined) {
    return false; // Asked to parse empty file
  }
  // Remove BOM if present
  if (data.charCodeAt(0) === 0xfeff) {
    data = data.substring(1);
  }
  if (!searchTokens) {
    const tree = parser.parse(data);
    walkTree(tree, fileItem, file);
    return false;
  } else {
    return false;
    let lines = data.split("\n");
    const semantics = new Semantics(
      lines,
      file,
      fileItem,
      itemsRepository,
      offset,
      range
    );
    semantics.analyze();
    return false;
  }
}

function walkTree(tree: TreeSitter.Tree, fileItem: FileItem, filePath: string) {
  for (let child of tree.rootNode.children) {
    if (child.type === "variable_declaration_statement") {
      readVariable(fileItem, child, filePath);
    }
  }
}
