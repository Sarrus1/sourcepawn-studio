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
const spParser = require("./spParser-gen");

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
  } else {
    data = fileItem.text;
  }
  parseText(data, file, fileItem, itemsRepository, searchTokens, IsBuiltIn);
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
    const args: spParserArgs = {
      fileItems: fileItem,
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
      const out = spParser.parse(data);
      return false;
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
          return true;
        }
        newOffset += offset;
        parseText(
          txt,
          file,
          fileItem,
          itemsRepository,
          searchTokens,
          isBuiltIn,
          newOffset
        );
      } else {
        console.error(err);
      }
      return false;
    }
  } else {
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
