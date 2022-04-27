import { Diagnostic, DiagnosticSeverity } from "vscode";
import { existsSync, readFileSync } from "fs";
import { resolve, dirname } from "path";
import { URI } from "vscode-uri";

import { ItemsRepository } from "../Backend/spItemsRepository";
import { FileItem } from "../Backend/spFilesRepository";
import { getNextScope, parsedLocToRange } from "./utils";
import { parserDiagnostics } from "../Providers/Linter/compilerDiagnostics";
import { ScoppedVariablesDeclaration } from "./interfaces";
import { Semantics } from "./Semantics/spSemantics";
const spParser = require("./spParser-gen");

export function parseFile(
  file: string,
  items: FileItem,
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
  parseText(data, file, items, itemsRepository, searchTokens, IsBuiltIn);
}

export interface spParserArgs {
  fileItems: FileItem;
  documents: Map<string, boolean>;
  filePath: string;
  IsBuiltIn: boolean;
  anonEnumCount: number;
  offset: number;
  variableDecl: ScoppedVariablesDeclaration;
}

export function parseText(
  data: string,
  file: string,
  items: FileItem,
  itemsRepository: ItemsRepository,
  searchTokens: boolean,
  isBuiltIn: boolean,
  offset: number = 0
) {
  if (data === undefined) {
    return; // Asked to parse empty file
  }
  // Remove BOM if present
  if (data.charCodeAt(0) === 0xfeff) {
    data = data.substring(1);
  }
  if (!searchTokens) {
    const args: spParserArgs = {
      fileItems: items,
      documents: itemsRepository.documents,
      filePath: file,
      IsBuiltIn: isBuiltIn,
      anonEnumCount: 0,
      offset,
      variableDecl: [],
    };
    if (offset === 0) {
      // Only clear the diagnostics if there is no error.
      parserDiagnostics.set(URI.file(file), []);
    }
    try {
      spParser.args = args;
      const out: string = spParser.parse(data);
      //console.debug(out);
    } catch (err) {
      if (err.location !== undefined) {
        const range = parsedLocToRange(err.location, args);
        const diagnostic = new Diagnostic(
          range,
          err.message,
          DiagnosticSeverity.Error
        );
        const newDiagnostics = Array.from(
          parserDiagnostics.get(URI.file(file))
        );
        newDiagnostics.push(diagnostic);
        parserDiagnostics.set(URI.file(file), newDiagnostics);
        let { txt, newOffset } = getNextScope(
          data,
          err.location.start.line - 1
        );
        if (txt === undefined || offset === undefined) {
          return;
        }
        newOffset += offset;
        parseText(
          txt,
          file,
          items,
          itemsRepository,
          searchTokens,
          isBuiltIn,
          newOffset
        );
      }
    }
  } else {
    const lines = data.split("\n");
    const semantics = new Semantics(lines, file, items, itemsRepository);
    semantics.analyze();
  }
}
