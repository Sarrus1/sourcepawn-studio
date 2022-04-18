import { CompletionItemKind, Diagnostic, DiagnosticSeverity } from "vscode";
import { existsSync, readFileSync } from "fs";
import { resolve, dirname } from "path";
import { URI } from "vscode-uri";

import { ItemsRepository } from "../Backend/spItemsRepository";
import { FileItems } from "../Backend/spFilesRepository";
import { SPItem } from "../Backend/Items/spItems";
import { handleReferenceInParser } from "./handleReferencesInParser";
import { getNextScope, parsedLocToRange, purgeCalls } from "./utils";
import { globalIdentifier } from "../Misc/spConstants";
import { FunctionItem } from "../Backend/Items/spFunctionItem";
import { MethodItem } from "../Backend/Items/spMethodItem";
import { PropertyItem } from "../Backend/Items/spPropertyItem";
import { MethodMapItem } from "../Backend/Items/spMethodmapItem";
import { EnumStructItem } from "../Backend/Items/spEnumStructItem";
import { parserDiagnostics } from "../Providers/Linter/compilerDiagnostics";
const spParser = require("./spParser2");

export function parseFile(
  file: string,
  items: FileItems,
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
  fileItems: FileItems;
  documents: Map<string, boolean>;
  filePath: string;
  IsBuiltIn: boolean;
  anonEnumCount: number;
  offset: number;
}

export function parseText(
  data: string,
  file: string,
  items: FileItems,
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
    };
    try {
      spParser.args = args;
      const out: string = spParser.parse(data);
      //console.debug(out);
      if (offset === 0) {
        // Only clear the diagnostics if there is no error.
        parserDiagnostics.set(URI.file(file), []);
      }
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
        let { txt, offset } = getNextScope(data, err.location.start.line - 1);
        if (txt === undefined || offset === undefined) {
          return;
        }
        parseText(
          txt,
          file,
          items,
          itemsRepository,
          searchTokens,
          isBuiltIn,
          offset
        );
      }
    }
  } else {
    const lines = data.split("\n");
    const parser = new Parser(lines, file, items, itemsRepository);
    parser.parse();
  }
}

export class Parser {
  fileItems: FileItems;
  items: SPItem[];
  lines: string[];
  lineNb: number;
  filePath: string;
  methodAndProperties: (MethodItem | PropertyItem)[];
  funcsAndMethodsInFile: (FunctionItem | MethodItem)[];
  MmEsInFile: (MethodMapItem | EnumStructItem)[];
  referencesMap: Map<string, SPItem>;

  constructor(
    lines: string[],
    filePath: string,
    completions: FileItems,
    itemsRepository: ItemsRepository
  ) {
    this.fileItems = completions;
    this.lineNb = 0;
    this.lines = lines;
    this.filePath = filePath;
    this.items = itemsRepository.getAllItems(URI.file(this.filePath));
    this.methodAndProperties = [];
    this.funcsAndMethodsInFile = [];
    this.MmEsInFile = [];
    this.referencesMap = new Map<string, SPItem>();
  }

  parse(): void {
    let line = this.lines[0];

    this.getReferencesMap();

    let lastFunc: FunctionItem | MethodItem | undefined;
    let lastMMorES: MethodMapItem | EnumStructItem | undefined;

    this.fileItems.tokens.sort((a, b) => {
      if (a.range.start.line === b.range.start.line) {
        return a.range.start.character - b.range.start.character;
      }
      return a.range.start.line - b.range.start.line;
    });
    const newDiagnostics = Array.from(
      parserDiagnostics.get(URI.file(this.filePath))
    );
    const thisArgs = {
      parser: this,
      offset: 0,
      previousItems: [],
      line,
      lineNb: 0,
      scope: "",
      outsideScope: "",
      allItems: this.items,
      filePath: this.filePath,
      diagnostics: newDiagnostics,
    };

    this.fileItems.tokens.forEach((e, i) => {
      const range = e.range;

      if (!lastFunc || !lastFunc.fullRange.contains(range)) {
        if (
          this.funcsAndMethodsInFile.length > 0 &&
          this.funcsAndMethodsInFile[0].fullRange.contains(range)
        ) {
          lastFunc = this.funcsAndMethodsInFile.shift();
        } else {
          lastFunc = undefined;
        }
      }

      if (!lastMMorES || !lastMMorES.fullRange.contains(range)) {
        if (
          this.MmEsInFile.length > 0 &&
          this.MmEsInFile[0].fullRange.contains(range)
        ) {
          lastMMorES = this.MmEsInFile.shift();
        } else {
          lastMMorES = undefined;
        }
      }
      const lineNb = range.start.line;

      if (lineNb !== thisArgs.lineNb || i === 0) {
        thisArgs.lineNb = range.start.line;
        thisArgs.line = this.lines[thisArgs.lineNb];
        thisArgs.offset = 0;
        thisArgs.previousItems = [];
        thisArgs.scope = `-${lastFunc ? lastFunc.name : globalIdentifier}-${
          lastMMorES ? lastMMorES.name : globalIdentifier
        }`;
        thisArgs.outsideScope = `-${globalIdentifier}-${
          lastMMorES ? lastMMorES.name : globalIdentifier
        }`;
      }
      try {
        handleReferenceInParser.call(thisArgs, e.id, range);
      } catch (err) {
        console.debug(err);
      }
      parserDiagnostics.set(URI.file(this.filePath), newDiagnostics);
    });
  }

  getReferencesMap(): void {
    const MC = [CompletionItemKind.Method, CompletionItemKind.Constructor];
    const MPC = [CompletionItemKind.Property].concat(MC);
    const MmEs = [CompletionItemKind.Class, CompletionItemKind.Struct];

    this.items.forEach((item, i) => {
      if (item.kind === CompletionItemKind.Variable) {
        purgeCalls(item, this.filePath);
        this.referencesMap.set(
          `${item.name}-${item.parent.name}-${
            item.parent.parent ? item.parent.parent.name : globalIdentifier
          }`,
          item
        );
      } else if (item.kind === CompletionItemKind.Function) {
        if (item.filePath === this.filePath) {
          this.funcsAndMethodsInFile.push(item as FunctionItem);
        }
        purgeCalls(item, this.filePath);
        this.referencesMap.set(item.name, item);
      } else if (MmEs.includes(item.kind)) {
        if (item.filePath === this.filePath) {
          this.MmEsInFile.push(item as MethodMapItem | EnumStructItem);
        }
        purgeCalls(item, this.filePath);
        this.referencesMap.set(item.name, item);
      } else if (!MPC.includes(item.kind) && item.references !== undefined) {
        purgeCalls(item, this.filePath);
        this.referencesMap.set(item.name, item);
      } else if (item.kind === CompletionItemKind.Property) {
        purgeCalls(item, this.filePath);
        this.methodAndProperties.push(item as PropertyItem);
      } else if (MC.includes(item.kind)) {
        if (item.filePath === this.filePath) {
          this.funcsAndMethodsInFile.push(item as MethodItem);
        }
        purgeCalls(item, this.filePath);
        this.methodAndProperties.push(item as MethodItem);
      }
    });

    this.MmEsInFile = this.MmEsInFile.sort(
      (a, b) => a.fullRange.start.line - b.fullRange.start.line
    );

    this.funcsAndMethodsInFile = this.funcsAndMethodsInFile.sort(
      (a, b) => a.fullRange.start.line - b.fullRange.start.line
    );
  }
}
