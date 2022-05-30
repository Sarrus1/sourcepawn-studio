import { CompletionItemKind, Range } from "vscode";
import { existsSync, readFileSync } from "fs";
import { resolve, dirname } from "path";

import { ItemsRepository } from "../Backend/spItemsRepository";
import { FileItem } from "../Backend/spFilesRepository";
import { Semantics } from "./Semantics/spSemantics";
import { PreProcessor } from "./PreProcessor/spPreprocessor";
import { parser } from "../spIndex";
import * as TreeSitter from "web-tree-sitter";
import { readVariable } from "./readVariable";
import { readFunctionAndMethod } from "./readFunctionAndMethod";
import { readEnum } from "./readEnum";
import { commentToDoc } from "./readDocumentation";
import { readDefine } from "./readDefine";
import { readEnumStruct } from "./readEnumStruct";
import { readMethodmap } from "./readMethodmap";
import { readTypedef } from "./readTypedef";
import { readTypeset } from "./readTypeset";

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
    const walker = new TreeWalker(fileItem, file, tree, isBuiltIn);
    walker.walkTree();
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

export class TreeWalker {
  fileItem: FileItem;
  filePath: string;
  tree: TreeSitter.Tree;
  isBuiltin: boolean;
  comments: TreeSitter.SyntaxNode[];
  anonEnumCount: number;
  deprecated: TreeSitter.SyntaxNode[];

  constructor(
    fileItem: FileItem,
    filePath: string,
    tree: TreeSitter.Tree,
    isBuiltin: boolean
  ) {
    this.fileItem = fileItem;
    this.filePath = filePath;
    this.tree = tree;
    this.isBuiltin = isBuiltin;
    this.comments = [];
    this.anonEnumCount = -1;
    this.deprecated = [];
  }

  public walkTree() {
    // TODO: Switch to a switch statement.
    for (let child of this.tree.rootNode.children) {
      if (child.type === "comment") {
        this.pushComment(child);
      }
      if (child.type === "preproc_pragma_deprecated") {
        this.deprecated.push(child);
      }
      if (
        child.type === "variable_declaration_statement" ||
        child.type === "old_variable_declaration_statement"
      ) {
        readVariable(this, child);
        continue;
      }
      if (
        child.type === "function_declaration" ||
        child.type === "function_definition" ||
        child.type === "callback_implementation"
      ) {
        readFunctionAndMethod(this, child);
        continue;
      }
      if (child.type === "enum") {
        readEnum(this, child);
        continue;
      }
      if (child.type === "preproc_define") {
        readDefine(this, child);
        continue;
      }
      if (child.type === "enum_struct") {
        readEnumStruct(this, child);
        continue;
      }
      if (child.type === "methodmap") {
        readMethodmap(this, child);
        continue;
      }
      if (child.type === "typedef") {
        readTypedef(this, child);
        continue;
      }
      if (child.type === "typeset") {
        readTypeset(this, child);
        continue;
      }
    }
  }

  /**
   * Process a comment and add it as a variable documentation if necessary.
   * @param  {TreeSitter.SyntaxNode} node   Node of the comment.
   * @returns void
   */
  public pushComment(node: TreeSitter.SyntaxNode): void {
    const lastItem = this.fileItem.items[this.fileItem.items.length - 1];
    const VaDe = [CompletionItemKind.Variable, CompletionItemKind.Constant];
    if (
      VaDe.includes(lastItem?.kind) &&
      lastItem.range.start.line === node.startPosition.row
    ) {
      lastItem.description += commentToDoc(node.text);
      return;
    }
    this.comments.push(node);
  }
}
