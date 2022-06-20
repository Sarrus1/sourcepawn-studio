import { CompletionItemKind, Diagnostic } from "vscode";
import { existsSync, readFileSync } from "fs";
import { resolve, dirname } from "path";
import { SyntaxNode, Tree } from "web-tree-sitter";

import { ItemsRepository } from "../Backend/spItemsRepository";
import { FileItem } from "../Backend/spFilesRepository";
import { Semantics } from "./Semantics/spSemantics";
import { PreProcessor } from "./PreProcessor/spPreprocessor";
import { parser, spLangObj, symbolQuery } from "../spIndex";
import { readVariable } from "./readVariable";
import { readFunctionAndMethod } from "./readFunctionAndMethod";
import { readEnum } from "./readEnum";
import { commentToDoc } from "./readDocumentation";
import { readDefine } from "./readDefine";
import { readEnumStruct } from "./readEnumStruct";
import { readMethodmap } from "./readMethodmap";
import { readTypedef } from "./readTypedef";
import { readTypeset } from "./readTypeset";
import { parserDiagnostics } from "../Providers/Linter/compilerDiagnostics";
import { URI } from "vscode-uri";
import { pointsToRange } from "./utils";
import { readMacro } from "./readMacro";

export function parseFile(
  file: string,
  fileItem: FileItem,
  itemsRepository: ItemsRepository,
  searchTokens: boolean
) {
  if (!existsSync(file)) {
    return;
  }
  let data = readFileSync(file, "utf-8");

  // Test for symbolic links
  const match = data.match(/^(?:\.\.\/)+(?:[/\w-])+\.\w+/);
  if (match !== null) {
    const folderpath = dirname(file);
    file = resolve(folderpath, match[0]);
    data = readFileSync(file, "utf-8");
  }
  if (!searchTokens) {
    if (fileItem.text === undefined) {
      const preprocessor = new PreProcessor(
        data.split("\n"),
        fileItem,
        itemsRepository
      );
      data = preprocessor.preProcess();
      fileItem.text = data;
    }
  }
  parseText(fileItem.text, file, fileItem, itemsRepository, searchTokens);
}

export function parseText(
  data: string,
  file: string,
  fileItem: FileItem,
  itemsRepository: ItemsRepository,
  searchTokens: boolean,
  offset: number = 0
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
    const walker = new TreeWalker(fileItem, file, tree);
    walker.walkTree();
    fileItem.symbols = symbolQuery.captures(tree.rootNode);
    parserDiagnostics.set(
      URI.file(file),
      spLangObj
        .query("(ERROR) @err")
        .captures(tree.rootNode)
        .map(
          (e) =>
            new Diagnostic(
              pointsToRange(e.node.startPosition, e.node.endPosition),
              e.node.text
            )
        )
    );
    return false;
  } else {
    const lines = data.split("\n");
    const semantics = new Semantics(
      lines,
      file,
      fileItem,
      itemsRepository,
      offset
    );
    semantics.analyze();
    return false;
  }
}

export class TreeWalker {
  fileItem: FileItem;
  filePath: string;
  tree: Tree;
  comments: SyntaxNode[];
  anonEnumCount: number;
  deprecated: SyntaxNode[];

  constructor(fileItem: FileItem, filePath: string, tree: Tree) {
    this.fileItem = fileItem;
    this.filePath = filePath;
    this.tree = tree;
    this.comments = [];
    this.anonEnumCount = -1;
    this.deprecated = [];
  }

  public walkTree() {
    for (const child of this.tree.rootNode.children) {
      switch (child.type) {
        case "comment":
          this.pushComment(child);
          break;
        case "preproc_pragma_deprecated":
          this.deprecated.push(child);
          break;
        case "global_variable_declaration":
        case "old_global_variable_declaration":
          readVariable(this, child);
          break;
        case "function_declaration":
        case "function_definition":
          readFunctionAndMethod(this, child);
          break;
        case "enum":
          readEnum(this, child);
          break;
        case "preproc_define":
          readDefine(this, child);
          break;
        case "enum_struct":
          readEnumStruct(this, child);
          break;
        case "methodmap":
          readMethodmap(this, child);
          break;
        case "typedef":
          readTypedef(this, child);
          break;
        case "typeset":
          readTypeset(this, child);
          break;
        case "preproc_macro":
          readMacro(this, child);
          break;
      }
    }
  }

  /**
   * Process a comment and add it as a variable documentation if necessary.
   * @param  {SyntaxNode} node   Node of the comment.
   * @returns void
   */
  public pushComment(node: SyntaxNode): void {
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
